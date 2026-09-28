use garde::Validate;
use serde::Deserialize;

use crate::web::validation::validate_hex_color;

#[derive(Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub(super) struct SaveProjectRequest {
    #[garde(length(bytes, min = 1, max = 255))]
    pub(super) name: String,
    pub(super) description: Option<String>,
    #[garde(inner(custom(validate_hex_color)))]
    pub(super) color: Option<String>,
}
