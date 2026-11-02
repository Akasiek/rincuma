use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Section {
    Projects,
    Tags,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SaveSection {
    pub(super) open: bool,
}
