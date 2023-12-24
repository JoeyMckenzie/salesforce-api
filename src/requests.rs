use serde::Deserialize;
use serde_json::Value;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateObjectRecordRequest {
    #[validate(required, length(min = 1))]
    object: Option<String>,
    #[validate(required)]
    fields: Option<String>
}