use serde_derive::Deserialize;
use serde_json::Value;

use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
enum PatroniState {
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "stopped")]
    Stopped,
}

impl std::string::ToString for PatroniState {
    fn to_string(&self) -> String {
        match self {
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
    pub fn postgres_version(&self) -> u32 { self.server_version }
    pub fn patroni_version(&self) -> &str { &self.patroni.version }

    pub fn status(&self) -> String { self.state.to_string() }
    pub fn pending_restart(&self) -> bool { self.pending_restart }
    pub fn is_master(&self) -> bool { self.role == PatroniRole::Master }
    pub fn is_running(&self) -> bool { self.state == PatroniState::Running }
    pub fn role(&self) -> String { self.role.to_string() }
    pub fn repl_slots(&self) -> usize { self.replication.len() }
    pub fn timeline(&self) -> u32 { self.timeline }
}
