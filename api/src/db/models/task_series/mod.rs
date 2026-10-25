mod model;

pub(crate) use model::TaskSeries;
#[cfg(test)]
pub(crate) use model::{SeriesEnd, SeriesFrequency};

#[cfg(test)]
mod tests;
