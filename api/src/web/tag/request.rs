use garde::Validate;
use serde::Deserialize;

use crate::web::validation::validate_hex_color;

#[derive(Deserialize, Validate)]
pub(super) struct SaveTagRequest {
    #[garde(length(bytes, min = 1, max = 255))]
    pub(super) name: String,
    #[garde(inner(custom(validate_hex_color)))]
    pub(super) color: Option<String>,
}
