/*use crate::patroni::PatroniStatus;

use hyper::{Client, Uri, Request};
use hyper::http::uri::Scheme;
use hyper_tls::HttpsConnector;

use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

use std::net::IpAddr;
use std::str::FromStr;
use base64::{encode, decode};

#[derive(Clone)]
pub struct EtcdClient {
    // client: Client<hyper::Body>,
    token: Option<String>,
    url: Uri,
}

#[derive(Serialize)]
struct KvRequest {
    key: Vec<u8>,
    range_end: Vec<u8>,
}

#[derive(Deserialize)]
struct KvResponse {
    kvs: Vec<Kv>,
}

#[derive(Deserialize)]
struct Kv {
    key: Vec<u8>,
    value: Vec<u8>
}

impl EtcdClient {
    pub fn new(url: &Uri) -> Self {
        Self {
            url: url.clone(),
            token: None,
        }
    }

    pub async fn service(&self, service: &str) -> Result<Vec<(String, PatroniStatus)>, Box<dyn std::error::Error>> {
        match self.url.scheme_str() {
            Some("https") => {
                let https = HttpsConnector::new();
                let client = Client::builder().build::<_, hyper::Body>(https);
                return self.use_client(service, client).await
            }
            _ => return self.use_client(service, Client::new()).await
        }
    }

    async fn use_client<T>(&self, service: &str, client: Client<T, hyper::Body>) -> Result<Vec<(String, PatroniStatus)>, Box<dyn std::error::Error>> {
        let services: KvResponse = {
            let url = format!("{}v3beta/kv/range", self.url);
            tracing::debug!(%url, "fetching service data");
            let key = format!("/{}", service);
            let kvRequest = KvRequest {
                key: key.as_bytes().to_owned(),
                range_end: "".as_bytes().to_owned(),
            };
            let req = Request::builder()
                .uri(url)
                .method(hyper::Method::POST)
                .body(serde_json::to_string(&kvRequest))
                .unwrap();
            let resp = Client::new().request(req).await?;
            let body = hyper::body::to_bytes(resp).await?;
            serde_json::from_slice(&body)?
        };
        Ok("hello".to_owned())
    }
}*/