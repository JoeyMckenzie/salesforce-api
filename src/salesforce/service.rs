use std::ops::Add;
use std::time::Duration;

use regex::Regex;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::Value;
use time::OffsetDateTime;
use tokio::sync::Mutex;
use tracing::info;

use crate::config::{SalesforceConfiguration, ServiceConfiguration};
use crate::errors::{ServiceError, ServiceResult};

#[derive(Debug)]
pub struct SalesforceService {
    http: reqwest::Client,
    config: SalesforceConfiguration,
    access_token: Mutex<Option<String>>,
    instance_url: Mutex<Option<String>>,
    expires_at: Mutex<OffsetDateTime>,
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
            access_token: Mutex::new(None),
            instance_url: Mutex::new(None),
            expires_at: Mutex::new(OffsetDateTime::UNIX_EPOCH),
        }
    }

    fn try_access_token(&self) -> ServiceResult<Option<String>> {
        match self.access_token.try_lock() {
            Ok(token_lock) => match token_lock.as_ref() {
                None => Err(ServiceError::AccessTokenNotFound),
                Some(token) => match self.try_refresh_required() {
                    Ok(refresh_required) => {
                        if refresh_required {
                            Ok(None)
                        } else {
                            Ok(Some(token.to_owned()))
                        }
                    }
                    Err(e) => Err(ServiceError::AuthenticationLockFailed(e.to_string())),
                },
            },
            Err(e) => Err(ServiceError::AuthenticationLockFailed(e.to_string())),
        }
    }

    fn try_refresh_required(&self) -> ServiceResult<bool> {
        match self.expires_at.try_lock() {
            Ok(expiration) => {
                let now = OffsetDateTime::now_utc();
                Ok(expiration.le(&now))
            }
            Err(e) => Err(ServiceError::AuthenticationLockFailed(e.to_string())),
        }
    }

    async fn get_access_token(&self) -> ServiceResult<String> {
        // If we have a cached access token, go ahead and grab it as it hasn't hit the expired time yet
        if let Ok(Some(cached_token)) = self.try_access_token() {
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
        dbg!(&token_response);
        let access_token = token_response.access_token;
        let instance_url = token_response.instance_url;

        if let Ok(mut token_lock) = self.access_token.try_lock() {
            *token_lock = Some(access_token.clone());
        }

        if let Ok(mut instance_url_lock) = self.instance_url.try_lock() {
            *instance_url_lock = Some(instance_url);
        }

        if let Ok(mut expiration_lock) = self.expires_at.try_lock() {
            let expires_in_duration = Duration::from_secs(60 * 30);
            *expiration_lock = OffsetDateTime::now_utc().add(expires_in_duration);
        }

        Ok(access_token)
    }

    pub async fn get_object_by_id(&self, object: String, id: String) -> ServiceResult<Value> {
        let access_token = self.get_access_token().await?;

        match self.instance_url.try_lock() {
            Ok(lock) => match lock.as_ref() {
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
            },
            Err(e) => Err(ServiceError::ObjectRetrievalFailed(e.to_string())),
        }
    }

    pub async fn get_objects(&self, soql: String) -> ServiceResult<Value> {
        let access_token = self.get_access_token().await?;

        match self.instance_url.try_lock() {
            Ok(lock) => match lock.as_ref() {
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
            },
            Err(e) => Err(ServiceError::ObjectRetrievalFailed(e.to_string())),
        }
    }
}

#[derive(Debug, Deserialize)]
struct AccessTokenResponse {
    pub access_token: String,
    pub instance_url: String,
    pub id: String,
    pub token_type: String,
    pub issued_at: String,
    pub signature: String,
}
