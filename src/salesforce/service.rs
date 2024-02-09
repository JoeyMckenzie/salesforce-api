use std::ops::Add;
use std::time::Duration;

use regex::Regex;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::Value;
use time::OffsetDateTime;
use tracing::info;

use crate::config::{SalesforceConfiguration, ServiceConfiguration};
use crate::errors::{ServiceError, ServiceResult};

#[derive(Debug, Clone)]
pub struct SalesforceService {
    http: reqwest::Client,
    config: SalesforceConfiguration,
    access_token: Option<String>,
    instance_url: Option<String>,
    expires_at: Option<OffsetDateTime>,
}

impl SalesforceService {
    pub fn new(
        salesforce_configuration: SalesforceConfiguration,
        service_configuration: ServiceConfiguration,
    ) -> Self {
        let timeout_duration =
            Duration::from_secs(service_configuration.timeout_seconds.unwrap_or(5));
        let client = reqwest::ClientBuilder::new()
            .timeout(timeout_duration)
            .build()
            .unwrap();
        Self {
            http: client,
            config: salesforce_configuration,
            access_token: None,
            instance_url: None,
            expires_at: None,
        }
    }

    fn check_access_token(&self) -> ServiceResult<Option<String>> {
        match &self.access_token {
            None => Err(ServiceError::AccessTokenNotFound),
            Some(token) => {
                if self.refresh_required() {
                    Ok(None)
                } else {
                    Ok(Some(token.to_owned()))
                }
            }
        }
    }

    fn refresh_required(&self) -> bool {
        match self.expires_at {
            None => true,
            Some(expiration) => {
                let now = OffsetDateTime::now_utc();
                expiration.le(&now)
            }
        }
    }

    async fn get_access_token(&mut self) -> ServiceResult<String> {
        // If we have a cached access token, go ahead and grab it as it hasn't hit the expired time yet
        if let Ok(Some(cached_token)) = self.check_access_token() {
            info!("Cached token found");
            return Ok(cached_token);
        }

        info!("No cached access token found, requesting a new one from Salesforce");

        let form = reqwest::multipart::Form::new()
            .text("grant_type", "password")
            .text("client_id", self.config.consumer_key.clone())
            .text("client_secret", self.config.consumer_secret.clone())
            .text("username", self.config.user_name.clone())
            .text("password", self.config.password.clone());

        let token_response = self
            .http
            .post(self.config.salesforce_url.clone())
            .multipart(form)
            .send()
            .await?
            .json::<AccessTokenResponse>()
            .await?;
        let access_token = token_response.access_token;
        let instance_url = token_response.instance_url;

        let expires_in_duration = Duration::from_secs(60 * 30);
        self.access_token = Some(access_token.clone());
        self.instance_url = Some(instance_url);
        self.expires_at = Some(OffsetDateTime::now_utc().add(expires_in_duration));

        Ok(access_token)
    }

    #[tracing::instrument]
    pub async fn get_object_by_id(&mut self, object: String, id: String) -> ServiceResult<Value> {
        let access_token = self.get_access_token().await?;

        match &self.instance_url {
            None => Err(ServiceError::InstanceUrlNotFound),
            Some(instance_url) => {
                let url = format!(
                    "{}/services/data/v59.0/sobjects/{}/{}",
                    instance_url, object, id
                );

                let response = self.http.get(&url).bearer_auth(access_token).send().await?;

                if response.status() == StatusCode::NOT_FOUND {
                    return Err(ServiceError::ObjectNotFound);
                }

                let object = response.json::<Value>().await?;

                Ok(object)
            }
        }
    }

    #[tracing::instrument]
    pub async fn get_objects(&mut self, soql: String) -> ServiceResult<Value> {
        let access_token = self.get_access_token().await?;

        match &self.instance_url {
            None => Err(ServiceError::InstanceUrlNotFound),
            Some(instance_url) => {
                let regex = Regex::new(r"\s+").unwrap();
                let updated_soql = regex.replace_all(&soql, " ").to_string();

                info!("Executing adjusted SOQL query:\n{updated_soql}");

                let url = format!(
                    "{}/services/data/v59.0/query/?q={}",
                    instance_url,
                    updated_soql.replace(' ', "+")
                );

                let response = self.http.get(&url).bearer_auth(access_token).send().await?;
                let objects = response.json::<Value>().await?;

                Ok(objects)
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct AccessTokenResponse {
    pub access_token: String,
    pub instance_url: String,
    pub token_type: String,
    pub issued_at: String,
    pub signature: String,
}
