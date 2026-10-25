use serde::{Deserialize, Deserializer, Serialize};

use crate::db::{SeriesEnd, SeriesFrequency};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::web::task) struct RecurrenceConfig {
    pub frequency: SeriesFrequency,
    pub interval: u32,
    pub time_zone: String,
    pub end: SeriesEnd,
}

#[derive(Default)]
pub(in crate::web::task) enum RecurrenceChange {
    #[default]
    Keep,
    Disable,
    Set(RecurrenceConfig),
}

impl<'de> Deserialize<'de> for RecurrenceChange {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let config = Option::<RecurrenceConfig>::deserialize(deserializer)?;
        Ok(config.map_or(Self::Disable, Self::Set))
    }
}
