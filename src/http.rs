use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use prometheus::*;

use std::net::SocketAddr;

pub async fn listen(listen_addr: SocketAddr) {
    let service = make_service_fn(|_| async {
        Ok::<_, hyper::Error>(service_fn(request_handler))
    });

    tracing::info!(%listen_addr, "starting HTTP server");
    Server::bind(&listen_addr)
        .serve(service).await
        .expect("Error starting HTTP server");
}

async fn request_handler(req: Request<Body>) -> Result<Response<Body>> {
    tracing::info!(method = %req.method(), path = %req.uri().path());

    match req.uri().path() {
        "/metrics" => {
            // let metrics = flowrider_common::metrics::to_text().unwrap_or_else(|_| String::from("# Error encoding metrics"));
            let encoder = TextEncoder::new();
            let metric_families = prometheus::gather();

            let mut buf = vec![];
            encoder.encode(&metric_families, &mut buf).unwrap();

            String::from_utf8(buf)
                .and_then(|metrics| Ok(Response::new(Body::from(metrics))))
                .or_else(|_| Ok(Response::new(Body::from("# Error encoding metrics"))))

        },

        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}