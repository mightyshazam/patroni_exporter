#[tokio::main]
async fn main() {
    patroni_exporter::run().await;
}