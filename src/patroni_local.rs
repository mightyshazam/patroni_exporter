use crate::patroni::{PatroniStatus, Exporter, ExporterResult, ExporterFuture};
use hyper::{Client, Uri};

#[derive(Clone)]
pub struct PatroniClient {
    url: Uri,
    host_name: String,
}

impl PatroniClient {
    pub fn new(url: &Uri, host_name: String) -> Self {
        Self {
            url: url.clone(),
            host_name: host_name.clone(),
        }
    }
}

impl Exporter for PatroniClient {
    fn name(&self) -> &'static str {
        "patroni"
    }

    fn service(
        & self,
        _: &str,
    ) -> ExporterFuture {
        let fut = service_async(self.url.clone(), self.host_name.clone());
        return ExporterFuture::new(Box::new(fut));
    }
}

async fn service_async(url: Uri, host_name: String) -> ExporterResult {
    let address = url.to_string();
    let client = Client::new();
    let res = match client.get(url).await {
        Ok(res) => res,
        Err(error) => {
            tracing::error!(%error, node = %address, "error fetching patroni state");
            return Ok(vec![])
        }
    };

    let bytes = match hyper::body::to_bytes(res).await {
        Ok(bytes) => bytes,
        Err(error) => {
            tracing::error!(%error, node = %address, "error reading stream");
            return Ok(vec![])
        }
    };

    let state: PatroniStatus = serde_json::from_slice(&bytes)?;
    Ok(vec![(host_name, state)])
}