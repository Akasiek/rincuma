//! Calendar schedules for task occurrences, independent of persistence and delivery.

use jiff::{
    Span, Timestamp,
    civil::{Date, DateTime},
    tz::TimeZone,
};
use std::num::NonZeroU32;

/// The calendar unit between scheduled occurrences.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
}

/// The mutually exclusive ways a task series can end.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum End {
    Never,
    /// Inclusive scheduled date in the series time zone.
    Through(Date),
    /// Total number of created occurrences, including the first task.
    After(u32),
}

/// Invalid schedule configuration or an unrepresentable calendar calculation.
#[derive(Debug, thiserror::Error)]
pub enum ScheduleError {
    #[error("The recurrence interval must be positive.")]
    InvalidInterval,
    #[error("The occurrence limit must be positive.")]
    InvalidLimit,
    #[error("The occurrence count must include the first task.")]
    InvalidCount,
    #[error("The occurrence limit cannot be lower than the created occurrence count.")]
    LimitBelowCount,
    #[error("The time zone must be an IANA time zone name.")]
    InvalidTimeZone,
    #[error(transparent)]
    Calendar(#[from] jiff::Error),
}

/// A series' calendar intent. Each slot is calculated from the original anchor.
#[derive(Clone, Debug)]
pub struct Schedule {
    frequency: Frequency,
    interval: NonZeroU32,
    time_zone: TimeZone,
    anchor: DateTime,
    end: End,
}

impl Schedule {
    /// Creates a schedule using a local calendar anchor and an IANA time zone.
    ///
    /// # Errors
    ///
    /// Rejects a zero interval, a zero occurrence limit, or an unknown time zone.
    pub fn new(
        frequency: Frequency,
        interval: u32,
        time_zone: &str,
        anchor: DateTime,
        end: End,
    ) -> Result<Self, ScheduleError> {
        let interval = NonZeroU32::new(interval).ok_or(ScheduleError::InvalidInterval)?;
        if end == End::After(0) {
            return Err(ScheduleError::InvalidLimit);
        }

        let time_zone = TimeZone::get(time_zone).map_err(|_| ScheduleError::InvalidTimeZone)?;
        if time_zone.iana_name().is_none() {
            return Err(ScheduleError::InvalidTimeZone);
        }

        Ok(Self {
            frequency,
            interval,
            time_zone,
            anchor,
            end,
        })
    }

    /// Finds the first slot strictly after both the original slot and completion.
    ///
    /// `scheduled_at` is the occurrence's original scheduled instant, never its
    /// manually edited due date. `created_count` counts tasks, not skipped slots.
    /// The caller must persist the successor and count atomically with completion.
    /// A gap moves the intended local time forward by the transition gap; a fold
    /// uses its earlier instant. Neither changes the anchor for subsequent slots.
    ///
    /// # Errors
    ///
    /// Rejects a zero count or a limit below the existing count. Returns a calendar
    /// error if the next slot exceeds the supported date or timestamp range.
    pub fn next_after(
        &self,
        scheduled_at: Timestamp,
        completed_at: Timestamp,
        created_count: u32,
    ) -> Result<Option<Timestamp>, ScheduleError> {
        if self.limit_reached(created_count)? {
            return Ok(None);
        }

        let cutoff = scheduled_at.max(completed_at);
        let mut index = self.estimated_index(cutoff)?;

        // A zone transition can move a slot onto a later local date. Check the
        // preceding slots before trusting the calendar estimate (e.g. Samoa 2011).
        while index > 1 && self.timestamp_at(index - 1)? > cutoff {
            index -= 1;
        }

        loop {
            let local = self.local_at(index)?;
            if self.past_end_date(local.date()) {
                return Ok(None);
            }

            let timestamp = self.time_zone.to_timestamp(local)?;
            if timestamp > cutoff {
                return Ok(Some(timestamp));
            }

            index += 1;
        }
    }

    fn limit_reached(&self, created_count: u32) -> Result<bool, ScheduleError> {
        if created_count == 0 {
            return Err(ScheduleError::InvalidCount);
        }

        let End::After(limit) = self.end else {
            return Ok(false);
        };
        if created_count > limit {
            return Err(ScheduleError::LimitBelowCount);
        }

        Ok(created_count == limit)
    }

    /// Estimates a slot index near `cutoff` to avoid scanning from the anchor.
    ///
    /// Converts `cutoff` to the series time zone, then divides the calendar
    /// distance from the anchor by the recurrence interval. Daily and weekly
    /// schedules use elapsed dates; monthly schedules use the year/month
    /// difference rather than a fixed number of days. The result is at least
    /// one because slot zero is the anchor, not a successor.
    ///
    /// For example, a two-day schedule anchored on October 10 estimates index
    /// two (October 14) for an October 15 cutoff. `next_after` then checks exact
    /// instants and advances to index three (October 16). This estimate ignores
    /// the time of day and zone transitions; `next_after` checks preceding slots
    /// as well, since a transition can shift a slot onto a later local date.
    ///
    /// # Errors
    ///
    /// Returns a calendar error if the date difference cannot be represented.
    fn estimated_index(&self, cutoff: Timestamp) -> Result<i64, ScheduleError> {
        let date = cutoff.to_zoned(self.time_zone.clone()).date();
        let days = i64::from(date.since(self.anchor.date())?.get_days());
        let units = match self.frequency {
            Frequency::Daily => days,
            Frequency::Weekly => days / 7,
            Frequency::Monthly => {
                (i64::from(date.year()) - i64::from(self.anchor.year())) * 12
                    + i64::from(date.month())
                    - i64::from(self.anchor.month())
            }
        };

        Ok((units / i64::from(self.interval.get())).max(1))
    }

    fn local_at(&self, index: i64) -> Result<DateTime, ScheduleError> {
        let units = index * i64::from(self.interval.get());
        let span = match self.frequency {
            Frequency::Daily => Span::new().try_days(units)?,
            Frequency::Weekly => Span::new().try_weeks(units)?,
            Frequency::Monthly => Span::new().try_months(units)?,
        };

        Ok(self.anchor.checked_add(span)?)
    }

    fn timestamp_at(&self, index: i64) -> Result<Timestamp, ScheduleError> {
        Ok(self.time_zone.to_timestamp(self.local_at(index)?)?)
    }

    fn past_end_date(&self, date: Date) -> bool {
        matches!(self.end, End::Through(end) if date > end)
    }
}
