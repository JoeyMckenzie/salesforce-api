use std::convert::TryFrom;

use axum::http::HeaderValue;

use crate::errors::ServiceError;

#[derive(Debug, Copy, Clone)]
pub enum SalesforceOrganization {
    NationalFunding,
    QuickBridge,
    Underwriting,
}

impl TryFrom<&HeaderValue> for SalesforceOrganization {
    type Error = ServiceError;

    fn try_from(header_value: &HeaderValue) -> Result<Self, Self::Error> {
        let org = header_value
            .to_str()
            .map_err(|e| ServiceError::InvalidOrganization(e.to_string()))?;

        let org_match = match org {
            "NationalFunding" => Self::NationalFunding,
            "QuickBridge" => Self::QuickBridge,
            "Underwriting" => Self::Underwriting,
            _ => {
                return Err(ServiceError::InvalidOrganization(format!(
                    "{org} is not a valid organization."
                )));
            }
        };

        Ok(org_match)
    }
}
