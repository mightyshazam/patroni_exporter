use hyper::Uri;
use lazy_static::lazy_static;
use prometheus::*;
use structopt::StructOpt;

mod consul;
mod etcd;
mod http;
mod patroni;
mod patroni_local;

use std::net::SocketAddr;
use std::time::Duration;
use patroni::Exporter;

const METHOD_CONSUL: &str = "consul";
const METHOD_ETCD: &str = "etcd";
const SERVICE_NONE: &str = "";
lazy_static! {
    pub static ref GAUGE_PG_VERSION: GaugeVec = register_gauge_vec!(
        "patroni_postgres_version",
        "PostgreSQL version",
        &["server"]
    )
    .unwrap();
    pub static ref GAUGE_PATRONI_VERSION: GaugeVec = register_gauge_vec!(
        "patroni_version",
        "Patroni version",
        &["server", "version"]
    )
    .unwrap();
    pub static ref GAUGE_RUNNING: GaugeVec = register_gauge_vec!(
        "patroni_running",
        "Is PostgreSQL running",
        &["server"]
    )
    .unwrap();
    pub static ref GAUGE_ROLE: GaugeVec = register_gauge_vec!(
        "patroni_role",
        "Patroni role",
        &["server", "role"]
    )
    .unwrap();
    pub static ref GAUGE_PENDING_RESTART: GaugeVec = register_gauge_vec!(
        "patroni_pending_restart",
        "Node is pending a restart",
        &["server"]
    )
    .unwrap();
    pub static ref GAUGE_TIMELINE: GaugeVec = register_gauge_vec!(
        "patroni_timeline_number",
        "Patroni timeline number",
        &["server"]
    )
    .unwrap();
    pub static ref GAUGE_REPL_SLOTS: GaugeVec = register_gauge_vec!(
        "patroni_replication_slots",
        "Postgres replication slots connected",
        &["server"]
    )
    .unwrap();
}

/// Export Patroni metrics to Prometheus
#[derive(StructOpt)]
#[structopt(name = "patroni_exporter")]
struct Args {
    /// Consul URL
    #[structopt(short = "c", long = "consul", env = "CONSUL_HTTP_ADDR", required_if("method", METHOD_CONSUL))]
    consul_url: Option<Uri>,

    /// Consul token
    #[structopt(short = "t", long = "token", env = "CONSUL_HTTP_TOKEN")]
    consul_token: Option<String>,

    /// Etcd URL
    #[structopt(short = "e", long = "etcd", env = "ETCD_HTTP_ADDR", required_if("method", METHOD_ETCD))]
    etcd_url: Option<Uri>,

    /// Patroni service name
    #[structopt(short = "s", long = "service", env = "PATRONI_SERVICE", required_if("method", METHOD_CONSUL), required_if("method", METHOD_ETCD))]
    service: Option<String>,

    /// HTTP listen address
    #[structopt(short = "l", long = "listen", default_value = "0.0.0.0:9393")]
    listen_addr: SocketAddr,

    /// Logging verbosity
    #[structopt(short = "v", parse(from_occurrences))]
    verbose: u8,

    // Local patroni endpoint
    #[structopt(short = "p", long = "patroni", env = "PATRONI_HTTP_ADDR", default_value = "http://127.0.0.1:8008")]
    local_addr: Uri,

    // Method to use for getting metrics
    #[structopt(short = "m", long = "method", env = "PATRONI_EXPORTER_METHOD")]
    method: String,

    #[structopt(short = "n", long = "name", env = "HOSTNAME")]
    name: String,
}

pub async fn run() {
    let args = Args::from_args();

    // Init logging
    // Derive verbosity from args
    let log_level = match args.verbose {
        0 => tracing::Level::INFO,
        1 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(log_level)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting tracing default failed");

    // Start HTTP server
    tokio::spawn(http::listen(args.listen_addr));

    let exporter: Box::<dyn Exporter> = match args.method.to_lowercase().trim() {
        METHOD_CONSUL => Box::new(consul::ConsulClient::new(&args.consul_url.unwrap(), &args.consul_token)),
        METHOD_ETCD => Box::new(etcd::EtcdClient::new(&args.etcd_url.unwrap())),
        _ => Box::new(patroni_local::PatroniClient::new(&args.local_addr, args.name))
    };

    // Keep track of Consul failures and bail after a few in a row
    let mut consul_fails = 0usize;

    // Also keep track of the nodes we're monitoring as these could change
    // HashMap<$node, $missing>

    let svc = match args.service {
        Some(service) => service,
        None => SERVICE_NONE.to_owned()
    };

    tracing::info!("Starting monitoring");
    loop {
        tracing::debug!("Querying Exporter");
        let fut = exporter.service(&svc);
        match fut.await {
            Ok(patroni) => {
                // Ensure we have fresh state
                GAUGE_ROLE.reset();
                GAUGE_PENDING_RESTART.reset();
                GAUGE_PATRONI_VERSION.reset();

                for (server, state) in &patroni {
                    let is_running = match state.is_running() {
                        true => 1.0,
                        false => 0.0,
                    };
                    GAUGE_RUNNING
                        .with_label_values(&[server])
                        .set(is_running);

                    GAUGE_ROLE
                        .with_label_values(&[server, &state.role()])
                        .set(1.0);

                    let pending_restart = match state.pending_restart() {
                        true => 1.0,
                        false => 0.0,
                    };
                    GAUGE_PENDING_RESTART
                        .with_label_values(&[server])
                        .set(pending_restart);

                    GAUGE_PG_VERSION
                        .with_label_values(&[server])
                        .set(state.postgres_version() as f64);

                    GAUGE_PATRONI_VERSION
                        .with_label_values(&[server, state.patroni_version()])
                        .set(1.0);

                    GAUGE_TIMELINE
                        .with_label_values(&[server])
                        .set(state.timeline() as f64);

                    GAUGE_REPL_SLOTS
                        .with_label_values(&[server])
                        .set(state.repl_slots() as f64);
                }

                // Reset our error counter
                consul_fails = 0;
            }
            Err(error) => {
                tracing::error!(%error, "unable to query service");
                consul_fails += 1;

                if consul_fails >= 5 {
                    tracing::error!("persistant error connecting to Consul, quitting");
                    break;
                }
            }
        };

        // Sleep for 10 secs
        tokio::time::delay_for(Duration::from_secs(30)).await;
    }
}