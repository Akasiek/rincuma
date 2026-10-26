use serde::{Deserialize, Serialize};

use super::{Frequency, ScheduleError};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Pattern {
    /// Weekdays use ISO numbering: Monday is 1 and Sunday is 7.
    Weekdays {
        days: Vec<u32>,
    },
    /// Ordinals 1..=5 select from the start; -1 selects the last weekday.
    MonthlyWeekday {
        ordinal: i32,
        weekday: u32,
    },
}

impl Pattern {
    pub(super) fn validate(&self, frequency: Frequency) -> Result<(), ScheduleError> {
        let valid = match self {
            Self::Weekdays { days } => is_valid_weekly_pattern(frequency, days),
            Self::MonthlyWeekday { ordinal, weekday } => {
                is_valid_monthly_pattern(frequency, *ordinal, *weekday)
            }
        };

        if !valid {
            return Err(ScheduleError::InvalidPattern);
        }
        Ok(())
    }
}

fn is_valid_weekly_pattern(frequency: Frequency, days: &[u32]) -> bool {
    frequency == Frequency::Weekly
        && !days.is_empty()
        && days.len() <= 7
        && days.iter().all(|day| (1..=7).contains(day))
        && has_unique_days(days)
}

fn is_valid_monthly_pattern(frequency: Frequency, ordinal: i32, weekday: u32) -> bool {
    frequency == Frequency::Monthly
        && (ordinal == -1 || (1..=5).contains(&ordinal))
        && (1..=7).contains(&weekday)
}

fn has_unique_days(days: &[u32]) -> bool {
    days.iter()
        .enumerate()
        .all(|(index, day)| !days.iter().take(index).any(|prior| prior == day))
}
