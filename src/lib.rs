use hyper::Uri;
use lazy_static::lazy_static;
use prometheus::*;
use structopt::StructOpt;
use tracing_subscriber::filter::EnvFilter;

mod consul;
mod http;
mod patroni;

use std::net::SocketAddr;
use std::time::Duration;

lazy_static! {
    pub static ref GAUGE_PG_VERSION: GaugeVec = register_gauge_vec!("patroni_postgres_version", "PostgreSQL version", &["node"]).unwrap();
    pub static ref GAUGE_PATRONI_VERSION: GaugeVec = register_gauge_vec!("patroni_version", "Patroni version", &["node", "version"]).unwrap();
    pub static ref GAUGE_RUNNING: GaugeVec = register_gauge_vec!("patroni_running", "Is PostgreSQL running", &["node"]).unwrap();
    pub static ref GAUGE_PENDING_RESTART: GaugeVec = register_gauge_vec!("patroni_pending_restart", "Node is pending a restart", &["node"]).unwrap();
    
    pub static ref GAUGE_TIMELINE: GaugeVec = register_gauge_vec!("patroni_timeline_number", "Patroni timeline number", &["node"]).unwrap();
    pub static ref GAUGE_ROLE: GaugeVec = register_gauge_vec!("patroni_role", "Current role", &["node", "role"]).unwrap();
    pub static ref GAUGE_REPL_SLOTS: GaugeVec = register_gauge_vec!("patroni_replication_slots", "Postgres replication slots connected", &["node"]).unwrap();
}

/// Export Patroni metrics to Prometheus
#[derive(StructOpt)]
#[structopt(name = "patroni_exporter")]
struct Args {
    /// Consul URL
    #[structopt(short = "c", long = "consul", env = "CONSUL_HTTP_ADDR")]
    consul_url: Uri,

    /// Consul token
    #[structopt(short = "t", long = "token", env = "CONSUL_HTTP_TOKEN")]
    consul_token: Option<String>,

    /// Patroni service name
    #[structopt(short = "s", long = "service", env = "PATRONI_SERVICE")]
    service: String,

    /// HTTP listen address
    #[structopt(short = "l", long = "listen")]
    listen_addr: SocketAddr,

    /// Logging verbosity
    #[structopt(short = "v", parse(from_occurrences))]
    verbose: u8
}

pub async fn run() {
    let args = Args::from_args();

    // Init logging
    // Derive verbosity from args
    let log_filter = match args.verbose {
        0 => format!("warn,{}=info", module_path!()),
        1 => format!("warn,{}=debug", module_path!()),
        2 => format!("info,{}=trace", module_path!()),
        _ => format!("debug,{}=trace", module_path!())
    };

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(EnvFilter::new(log_filter))
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting tracing default failed");

    // Start HTTP server
    tokio::spawn(http::listen(args.listen_addr));
    
    let consul = consul::ConsulClient::new(&args.consul_url, &args.consul_token);
    
    // Keep track of Consul failures and bail after a few in a row
    let mut consul_fails = 0usize;

    // Also keep track of the nodes we're monitoring as these could change
    // HashMap<$node, $missing>

    tracing::info!("Starting monitoring");
    loop {
        tracing::debug!("Querying Consul");
        match consul.service(&args.service).await {
            Ok(patroni) => {
                for (node, state) in &patroni {
                    let is_running = match state.is_running() {
                        true => 1.0,
                        false => 0.0
                    };
                    GAUGE_RUNNING.with_label_values(&[node]).set(is_running);

                    let pending_restart = match state.pending_restart() {
                        true => 1.0,
                        false => 0.0
                    };
                    GAUGE_PENDING_RESTART.with_label_values(&[node]).set(pending_restart);
                    
                    GAUGE_PG_VERSION.with_label_values(&[node]).set(state.postgres_version() as f64);
                    GAUGE_PATRONI_VERSION.with_label_values(&[node, state.patroni_version()]).set(1.0);
                    
                    GAUGE_TIMELINE.with_label_values(&[node]).set(state.timeline() as f64);
                    GAUGE_ROLE.with_label_values(&[node, &state.role()]).set(1.0);
                    GAUGE_REPL_SLOTS.with_label_values(&[node]).set(state.repl_slots() as f64);
                }

                // Reset our error counter
                consul_fails = 0;
            },
            Err(error) => {
                tracing::error!(%error, "unable to query consul");
                consul_fails += 1;

                if consul_fails >= 5 {
                    tracing::error!("persistant error connecting to Consul, quitting");
                    break;
                }
            }
        };

        // Sleep for 10 secs
        tokio::timer::delay_for(Duration::from_secs(30)).await;
    }
}