use serde::Deserialize;
use serde_json::Value;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateObjectRecordRequest {
    #[validate(required)]
    fields: Option<Value>,
}
