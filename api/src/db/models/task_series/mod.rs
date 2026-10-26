mod model;
mod pattern;

pub(crate) use model::{SeriesEnd, SeriesFrequency, TaskSeries};
pub(crate) use pattern::SeriesPattern;

#[cfg(test)]
mod tests;
