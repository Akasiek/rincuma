#![allow(
    clippy::unused_self,
    reason = "Toasty generates enum variant field accessors that do not use self."
)]

use toasty::Embed;

use crate::recurrence::Pattern;

/// Compact persisted form of the validated calendar pattern.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Embed)]
pub(crate) enum SeriesPattern {
    Weekdays { mask: u32 },
    MonthlyWeekday { ordinal: i32, weekday: u32 },
}

impl From<&Pattern> for SeriesPattern {
    fn from(pattern: &Pattern) -> Self {
        match pattern {
            Pattern::Weekdays { days } => Self::Weekdays {
                mask: days.iter().fold(0, |mask, day| mask | (1 << (day - 1))),
            },
            Pattern::MonthlyWeekday { ordinal, weekday } => Self::MonthlyWeekday {
                ordinal: *ordinal,
                weekday: *weekday,
            },
        }
    }
}

impl From<SeriesPattern> for Pattern {
    fn from(pattern: SeriesPattern) -> Self {
        match pattern {
            SeriesPattern::Weekdays { mask } => Self::Weekdays {
                days: (1..=7).filter(|day| mask & (1 << (day - 1)) != 0).collect(),
            },
            SeriesPattern::MonthlyWeekday { ordinal, weekday } => {
                Self::MonthlyWeekday { ordinal, weekday }
            }
        }
    }
}
