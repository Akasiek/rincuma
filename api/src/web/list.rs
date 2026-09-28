use serde::{Deserialize, Serialize};

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 100;

#[derive(Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub(super) enum SortOrder {
    Asc,
    #[default]
    Desc,
}

pub(super) struct Page {
    pub(super) limit: usize,
    pub(super) offset: usize,
}

impl Page {
    pub(super) fn new(limit: Option<usize>, offset: Option<usize>) -> Option<Self> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT);
        let offset = offset.unwrap_or(0);
        if !(1..=MAX_LIMIT).contains(&limit) || i64::try_from(offset).is_err() {
            return None;
        }

        Some(Self { limit, offset })
    }
}

#[derive(Serialize)]
pub(super) struct PageResponse<T> {
    pub(super) items: Vec<T>,
    pub(super) total: u64,
    pub(super) limit: usize,
    pub(super) offset: usize,
}

impl<T> PageResponse<T> {
    pub(super) fn new(items: Vec<T>, total: u64, page: &Page) -> Self {
        Self {
            items,
            total,
            limit: page.limit,
            offset: page.offset,
        }
    }
}
