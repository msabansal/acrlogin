use std::sync::Arc;

use crate::Result;
use azure_core::auth::TokenCredential;
use reqwest::{Client, ClientBuilder};
use serde::Deserialize;

pub struct AcrClient {
    credentials: Arc<dyn TokenCredential>,
    endpoint: String,
    client: Client,
    scope: String,
    tenant: String,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct AcrTokenResponse {
    pub refresh_token: String,
}

impl AcrClient {
    pub fn new(
        endpoint: &str,
        tenant: &str,
        credentials: Arc<dyn TokenCredential>,
    ) -> Result<AcrClient> {
        let client = ClientBuilder::default().build()?;
        let scope = "https://management.core.windows.net/".to_string();
        Ok(AcrClient {
            credentials,
            endpoint: endpoint.to_string(),
            client,
            scope,
            tenant: tenant.to_string(),
        })
    }

    pub async fn get_access_token(&self) -> Result<AcrTokenResponse> {
        let token_response = self.credentials.get_token(&self.scope).await?;

        let data = vec![
            ("grant_type", "access_token".to_string()),
            ("service", self.endpoint.to_string()),
            ("access_token", token_response.token.secret().to_string()),
            ("tenant", self.tenant.to_string()),
        ];

        let response = self
            .client
            .post(format!("https://{}/oauth2/exchange", self.endpoint))
            .header(
                http::header::AUTHORIZATION,
                format!("Bearer {}", token_response.token.secret()),
            )
            .form(&data)
            .send()
            .await?
            .json::<AcrTokenResponse>()
            .await?;
        Ok(response)
    }
}
