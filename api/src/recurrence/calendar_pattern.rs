use jiff::{Span, Timestamp, civil::Date};

use super::{Pattern, Schedule, ScheduleError};

impl Schedule {
    pub(super) fn next_pattern(
        &self,
        pattern: &Pattern,
        cutoff: Timestamp,
    ) -> Result<Option<Timestamp>, ScheduleError> {
        match pattern {
            Pattern::Weekdays { days } => self.next_weekdays(days, cutoff),
            Pattern::MonthlyWeekday { ordinal, weekday } => {
                self.next_monthly_weekday(*ordinal, *weekday, cutoff)
            }
        }
    }

    fn next_weekdays(
        &self,
        days: &[u32],
        cutoff: Timestamp,
    ) -> Result<Option<Timestamp>, ScheduleError> {
        let anchor_date = self.anchor.date();
        let monday = anchor_date.checked_sub(
            Span::new().days(i64::from(anchor_date.weekday().to_monday_zero_offset())),
        )?;
        let cutoff_date = cutoff.to_zoned(self.time_zone.clone()).date();
        let elapsed_days = i64::from(cutoff_date.since(monday)?.get_days());
        let interval = i64::from(self.interval.get());
        let first_cycle = (elapsed_days / (7 * interval)).saturating_sub(1).max(0);
        let mut days = days.to_vec();
        days.sort_unstable();

        self.find_next_weekday_slot(monday, &days, first_cycle, cutoff)
    }

    fn find_next_weekday_slot(
        &self,
        monday: Date,
        days: &[u32],
        mut cycle: i64,
        cutoff: Timestamp,
    ) -> Result<Option<Timestamp>, ScheduleError> {
        let interval = i64::from(self.interval.get());
        loop {
            let week = monday.checked_add(Span::new().try_weeks(cycle * interval)?)?;
            for day in days {
                let date = week.checked_add(Span::new().days(i64::from(*day) - 1))?;
                if self.past_end_date(date) {
                    return Ok(None);
                }

                let timestamp = self
                    .time_zone
                    .to_timestamp(date.to_datetime(self.anchor.time()))?;
                if timestamp > cutoff {
                    return Ok(Some(timestamp));
                }
            }
            cycle += 1;
        }
    }

    fn next_monthly_weekday(
        &self,
        ordinal: i32,
        weekday: u32,
        cutoff: Timestamp,
    ) -> Result<Option<Timestamp>, ScheduleError> {
        let anchor_month = self.anchor.date().first_of_month();
        let cutoff_date = cutoff.to_zoned(self.time_zone.clone()).date();
        let months = (i64::from(cutoff_date.year()) - i64::from(anchor_month.year())) * 12
            + i64::from(cutoff_date.month())
            - i64::from(anchor_month.month());
        let interval = i64::from(self.interval.get());
        let first_cycle = (months / interval).saturating_sub(1).max(0);

        self.find_next_monthly_weekday_slot(anchor_month, first_cycle, ordinal, weekday, cutoff)
    }

    fn find_next_monthly_weekday_slot(
        &self,
        anchor_month: Date,
        first_cycle: i64,
        ordinal: i32,
        weekday: u32,
        cutoff: Timestamp,
    ) -> Result<Option<Timestamp>, ScheduleError> {
        let interval = i64::from(self.interval.get());
        // The Gregorian weekday pattern repeats every 400 years. If an interval
        // never reaches a month with a fifth weekday, the series has no successor.
        for cycle in first_cycle..=first_cycle + month_pattern_period(interval) {
            let month = anchor_month.checked_add(Span::new().try_months(cycle * interval)?)?;
            if self.past_end_date(month) {
                return Ok(None);
            }

            if let Some(date) = ordinal_weekday(month, ordinal, weekday)? {
                if self.past_end_date(date) {
                    return Ok(None);
                }
                let timestamp = self
                    .time_zone
                    .to_timestamp(date.to_datetime(self.anchor.time()))?;
                if timestamp > cutoff {
                    return Ok(Some(timestamp));
                }
            }
        }
        Ok(None)
    }
}

fn ordinal_weekday(month: Date, ordinal: i32, weekday: u32) -> Result<Option<Date>, ScheduleError> {
    let start = if ordinal == -1 {
        month.last_of_month()
    } else {
        month
    };

    let start_weekday = i64::from(start.weekday().to_monday_one_offset());
    let weekday = i64::from(weekday);

    let offset = if ordinal == -1 {
        -(start_weekday - weekday).rem_euclid(7)
    } else {
        (weekday - start_weekday).rem_euclid(7) + 7 * (i64::from(ordinal) - 1)
    };
    let date = start.checked_add(Span::new().days(offset))?;

    Ok((date.month() == month.month()).then_some(date))
}

// A monthly interval visits only this many distinct positions in the 400-year
// Gregorian calendar cycle. Bound the search when a fifth weekday never occurs.
fn month_pattern_period(interval: i64) -> i64 {
    let mut left = 4800;
    let mut right = interval;
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    4800 / left
}
