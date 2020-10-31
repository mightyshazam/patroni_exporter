use crate::patroni::PatroniStatus;

use hyper::{Client, Uri};
use serde_derive::Deserialize;
use uuid::Uuid;

use std::net::IpAddr;
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ConsulService {
    #[serde(rename = "ID")]
    id: Uuid,
    node: String,
    datacenter: String,
    service_address: IpAddr,
    service_port: u16,
}

#[derive(Clone)]
pub struct ConsulClient {
    // client: Client<hyper::Body>,
    token: Option<String>,
    url: Uri,
}

impl ConsulClient {
    pub fn new(url: &Uri, token: &Option<String>) -> Self {
        Self {
            // client: Client::new(),
            token: token.clone(),
            url: url.clone(),
        }
    }

    pub async fn service(
        &self,
        service: &str,
    ) -> Result<Vec<(String, PatroniStatus)>, Box<dyn std::error::Error>> {
        let client = Client::new();
        let services: Vec<ConsulService> = {
            let url = format!("{}v1/catalog/service/{}", self.url, service);
            tracing::debug!(%url, "fetching service data");
            let res = client.get(Uri::from_str(&url)?).await?;
            let body = hyper::body::to_bytes(res).await?;
            // let bytes = res.into_body().await?;

            serde_json::from_slice(&body)?
        };
        tracing::trace!(?service);

        let mut status = vec![];
        for service in &services {
            let url = format!(
                "http://{}:{}/",
                service.service_address, service.service_port
            );
            tracing::debug!(%url, "fetching patroni state");

            let res = match client.get(Uri::from_str(&url)?).await {
                Ok(res) => res,
                Err(error) => {
                    tracing::error!(%error, node = %service.service_address, "error fetching patroni state");
                    continue;
                }
            };

            let bytes = match hyper::body::to_bytes(res).await {
                Ok(bytes) => bytes,
                Err(error) => {
                    tracing::error!(%error, node = %service.service_address, "error reading stream");
                    continue;
                }
            };

            let state: PatroniStatus = serde_json::from_slice(&bytes)?;
            status.push((service.node.clone(), state));
        }

        Ok(status)
    }
}
