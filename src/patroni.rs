use serde_derive::Deserialize;
use serde_json::Value;

use std::collections::HashMap;
use std::net::IpAddr;
use std::pin::Pin;
use std::future::Future;
use std::task::{Context, Poll};

pub type ExporterResult = Result<Vec<(String, PatroniStatus)>, Box<dyn std::error::Error>>;

pub struct ExporterFuture {
    inner: Pin<Box<dyn Future<Output = ExporterResult> + Send>>,
}

impl ExporterFuture {
    pub fn new(fut: Box<dyn Future<Output = ExporterResult> + Send>) -> Self {
        Self { inner: fut.into() }
    }
}

/*
impl fmt::Debug for ResponseFExporterFutureuture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("Future<Response>")
    }
}*/

impl Future for ExporterFuture {
    type Output = ExporterResult;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx)
    }
}

pub trait Exporter {
    fn name(&self) -> &'static str;
    fn service(
        &self,
        service: & str,
    ) -> ExporterFuture;
}



#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
enum PatroniState {
    #[serde(rename = "starting")]
    Starting,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "stopped")]
    Stopped,
}

impl std::string::ToString for PatroniState {
    fn to_string(&self) -> String {
        match self {
            PatroniState::Starting => "starting".into(),
            PatroniState::Running => "running".into(),
            PatroniState::Stopped => "stopped".into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
enum PatroniRole {
    #[serde(rename = "master")]
    Master,
    #[serde(rename = "replica")]
    Replica,
}

impl std::string::ToString for PatroniRole {
    fn to_string(&self) -> String {
        match self {
            PatroniRole::Master => "master".into(),
            PatroniRole::Replica => "replica".into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct PatroniReplConn {
    usename: String,
    client_addr: IpAddr,
    state: String,
    sync_state: String,
}

#[derive(Clone, Debug, Deserialize)]
struct PatroniInfo {
    version: String,
    scope: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PatroniStatus {
    #[serde(default)]
    pending_restart: bool,
    state: PatroniState,
    role: PatroniRole,
    server_version: u32,
    cluster_unlocked: bool,
    timeline: u32,
    // lazy..
    xlog: HashMap<String, Value>,
    #[serde(default)]
    replication: Vec<PatroniReplConn>,
    patroni: PatroniInfo,
}

impl PatroniStatus {
    pub fn postgres_version(&self) -> u32 {
        self.server_version
    }
    pub fn patroni_version(&self) -> &str {
        &self.patroni.version
    }

    pub fn status(&self) -> String {
        self.state.to_string()
    }
    pub fn pending_restart(&self) -> bool {
        self.pending_restart
    }
    pub fn is_master(&self) -> bool {
        self.role == PatroniRole::Master
    }
    pub fn is_running(&self) -> bool {
        self.state == PatroniState::Running
    }
    pub fn role(&self) -> String {
        self.role.to_string()
    }
    pub fn repl_slots(&self) -> usize {
        self.replication.len()
    }
    pub fn timeline(&self) -> u32 {
        self.timeline
    }
}
