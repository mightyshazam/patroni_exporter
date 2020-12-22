use crate::patroni::{PatroniStatus, Exporter, ExporterResult, ExporterFuture};

use hyper::{Client, Uri, Request};
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone)]
pub struct EtcdClient {
    // client: Client<hyper::Body>,
    token: Option<String>,
    url: Uri,
}

#[derive(Serialize)]
struct KvRequest {
    #[serde(with = "base64")]
    key: Vec<u8>,
    #[serde(with = "base64")]
    range_end: Vec<u8>,
}

#[derive(Deserialize)]
struct KvResponse {
    kvs: Option<Vec<Kv>>,
}

#[derive(Deserialize)]
struct Kv {
    #[serde(with = "base64")]
    key: Vec<u8>,
    #[serde(with = "base64")]
    value: Vec<u8>
}

#[derive(Deserialize)]
struct EtcdServiceEntry {
    api_url: String,
}

impl EtcdClient {
    pub fn new(url: &Uri) -> Self {
        Self {
            url: url.clone(),
            token: None,
        }
    }
}

impl Exporter for EtcdClient {
    fn name(&self) -> &'static str {
        "etcd"
    }

    fn service(
        & self,
        service: &str,
    ) -> ExporterFuture {
        let fut = service_async(self.url.clone(), String::from(service));
        return ExporterFuture::new(Box::new(fut));
    }
}

async fn service_async(uri: Uri, service: String) -> ExporterResult {
    let client = Client::new();
    let url = format!("{}v3beta/kv/range", uri);
    tracing::debug!(%url, "fetching service data");
    let key = format!("/service/{}/members", service);
    let range_end = format!("/service/{}/membersz", service);
    let kv_request = KvRequest {
        key: key.as_bytes().to_owned(),
        range_end: range_end.as_bytes().to_owned(),
    };
    let req_string = serde_json::to_string(&kv_request).unwrap();
    let req = Request::builder()
        .uri(url.clone())
        .method(hyper::Method::POST)
        .body(hyper::Body::from(req_string))
        .unwrap();
    let res = match client.request(req).await {
        Ok(res) => res,
        Err(error) => {
            tracing::error!(%error, node = %url, "error fetching patroni state");
            return Ok(vec![])
        }
    };

    let body = hyper::body::to_bytes(res).await?;
    let items: KvResponse = serde_json::from_slice(&body)?;

    let mut status = vec![];
    let kvs_found = match items.kvs {
        Some(f) => f,
        None => {
            tracing::warn!("no services found");
            return Ok(vec![])
        }
    };
    for kv in kvs_found {
        let str_key = match String::from_utf8(kv.key) {
            Ok(str_key) => str_key,
            Err(error) => {
                tracing::error!(%error, "error decoding key");
                continue
            },
        };

        let entry:EtcdServiceEntry = serde_json::from_slice(&kv.value).unwrap();
        let res = match client.get(Uri::from_str(entry.api_url.as_str())?).await {
            Ok(res) => res,
            Err(error) => {
                tracing::error!(%error, node = %entry.api_url, "error fetching patroni state");
                continue
            }
        };

        let bytes = match hyper::body::to_bytes(res).await {
            Ok(bytes) => bytes,
            Err(error) => {
                tracing::error!(%error, node = %entry.api_url, "error reading stream");
                continue
            }
        };

        let state: PatroniStatus = match serde_json::from_slice(&bytes) {
            Ok(state) => state,
            Err(error) => {
                tracing::error!(%error, node = %entry.api_url, "unable to retrieve state");
                continue
            }
        };
        match str_key.split('/').last() {
            Some(name) => {
                status.push((name.to_string(), state))
            }
            _ => status.push((entry.api_url, state))
        }
    }

    Ok(status)
}

mod base64 {
    extern crate base64;
    use serde::{Serializer, de, Deserialize, Deserializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&base64::encode(bytes))

        // Could also use a wrapper type with a Display implementation to avoid
        // allocating the String.
        //
        // serializer.collect_str(&Base64(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
        where D: Deserializer<'de>
    {
        let s = <&str>::deserialize(deserializer)?;
        base64::decode(s).map_err(de::Error::custom)
    }
}