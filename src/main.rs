use std::env;
use std::sync::Arc;

use crate::config::Config;
use crate::metrics::{register, register_int, FloatGauge, IntGauge};
use crate::speedtest::{run_speedtest, SpeedtestResult};
use axum::routing::get;
use axum::Router;
use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use clap::Parser;
use dotenv::dotenv;
use log::{debug, error, info};
use prometheus::{Encoder, Registry, TextEncoder};
use serde::Deserialize;
use tokio::task::spawn_blocking;

mod config;
mod metrics;
mod speedtest;

struct AppState {
    registry: Registry,
}

impl AppState {
    fn new() -> Self {
        Self {
            registry: Registry::new(),
        }
    }
}

struct RequestState {
    registry: Registry,

    ping_latency_gauge: FloatGauge,
    ping_low_gauge: FloatGauge,
    ping_high_gauge: FloatGauge,

    download_bytes_gauge: IntGauge,
    download_bandwidth_bytes_gauge: IntGauge,
    download_elapsed_seconds_gauge: FloatGauge,
    download_latency_iqm_seconds_gauge: FloatGauge,
    download_latency_low_seconds_gauge: FloatGauge,
    download_latency_high_seconds_gauge: FloatGauge,

    upload_bytes_gauge: IntGauge,
    upload_bandwidth_bytes_gauge: IntGauge,
    upload_elapsed_seconds_gauge: FloatGauge,
    upload_latency_iqm_seconds_gauge: FloatGauge,
    upload_latency_low_seconds_gauge: FloatGauge,
    upload_latency_high_seconds_gauge: FloatGauge,
}

impl RequestState {
    fn new() -> Self {
        let registry = Registry::new();
        Self {
            ping_latency_gauge: register(
                &registry,
                "speedtest_ping_latency_seconds",
                "Speedtest ping latency in seconds",
            ),
            ping_low_gauge: register(
                &registry,
                "speedtest_ping_low_seconds",
                "Speedtest ping low in seconds",
            ),
            ping_high_gauge: register(
                &registry,
                "speedtest_ping_high_seconds",
                "Speedtest ping high in seconds",
            ),

            download_bytes_gauge: register_int(
                &registry,
                "speedtest_download_bytes",
                "Number of bytes downloaded during speedtest",
            ),
            download_bandwidth_bytes_gauge: register_int(
                &registry,
                "speedtest_download_bandwidth_bytes",
                "Speedtest download bandwidth in bytes per second",
            ),
            download_elapsed_seconds_gauge: register(
                &registry,
                "speedtest_download_elapsed_seconds",
                "Speedtest download elapsed time in seconds",
            ),
            download_latency_iqm_seconds_gauge: register(
                &registry,
                "speedtest_download_latency_iqm_seconds",
                "Speedtest download latency iqm in seconds",
            ),
            download_latency_low_seconds_gauge: register(
                &registry,
                "speedtest_download_latency_low_seconds",
                "Speedtest download latency low in seconds",
            ),
            download_latency_high_seconds_gauge: register(
                &registry,
                "speedtest_download_latency_high_seconds",
                "Speedtest download latency high in seconds",
            ),

            upload_bytes_gauge: register_int(
                &registry,
                "speedtest_upload_bytes",
                "Number of bytes uploaded during speedtest",
            ),
            upload_bandwidth_bytes_gauge: register_int(
                &registry,
                "speedtest_upload_bandwidth_bytes",
                "Speedtest upload bandwidth in bytes per second",
            ),
            upload_elapsed_seconds_gauge: register(
                &registry,
                "speedtest_upload_elapsed_seconds",
                "Speedtest upload elapsed time in seconds",
            ),
            upload_latency_iqm_seconds_gauge: register(
                &registry,
                "speedtest_upload_latency_iqm_seconds",
                "Speedtest upload latency iqm in seconds",
            ),
            upload_latency_low_seconds_gauge: register(
                &registry,
                "speedtest_upload_latency_low_seconds",
                "Speedtest upload latency low in seconds",
            ),
            upload_latency_high_seconds_gauge: register(
                &registry,
                "speedtest_upload_latency_high_seconds",
                "Speedtest upload latency high in seconds",
            ),

            registry,
        }
    }

    fn set(&self, result: &SpeedtestResult) {
        self.ping_latency_gauge
            .set(result.ping.latency_seconds(), result);
        self.ping_low_gauge.set(result.ping.low_seconds(), result);
        self.ping_high_gauge
            .set(result.ping.high_seconds(), result);

        self.download_bytes_gauge.set(result.download.bytes, result);
        self.download_bandwidth_bytes_gauge
            .set(result.download.bandwidth, result);
        self.download_elapsed_seconds_gauge
            .set(result.download.elapsed_seconds(), result);
        self.download_latency_iqm_seconds_gauge
            .set(result.download.latency_iqm_seconds(), result);
        self.download_latency_low_seconds_gauge
            .set(result.download.latency_low_seconds(), result);
        self.download_latency_high_seconds_gauge
            .set(result.download.latency_high_seconds(), result);

        self.upload_bytes_gauge.set(result.upload.bytes, result);
        self.upload_bandwidth_bytes_gauge
            .set(result.upload.bandwidth, result);
        self.upload_elapsed_seconds_gauge
            .set(result.upload.elapsed_seconds(), result);
        self.upload_latency_iqm_seconds_gauge
            .set(result.upload.latency_iqm_seconds(), result);
        self.upload_latency_low_seconds_gauge
            .set(result.upload.latency_low_seconds(), result);
        self.upload_latency_high_seconds_gauge
            .set(result.upload.latency_high_seconds(), result);
    }

    fn encode(&self) -> Vec<u8> {
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();
        encoder
            .encode(&self.registry.gather(), &mut buffer)
            .unwrap();
        buffer
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }
    pretty_env_logger::init();

    let config = Config::parse();
    debug!("Loaded configuration: {:?}", config);

    let addr = format!("{}:{}", config.http_host, config.http_port);
    let listener = tokio::net::TcpListener::bind(addr.clone()).await.unwrap();

    let app_state = Arc::new(AppState::new());

    let app = Router::new()
        .route("/metrics", get(handle_metrics))
        .route("/speedtest", get(handle_speedtest))
        .with_state(app_state);

    info!("🦀Server running at http://{}", &addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.unwrap();
        })
        .await
        .unwrap();
}

#[derive(Deserialize)]
pub struct SpeedtestQuery {
    server_id: String,
}

async fn handle_speedtest(
    State(_app_state): State<Arc<AppState>>, // reserved for future app-level metrics
    Query(params): Query<SpeedtestQuery>,
) -> impl IntoResponse {
    if params.server_id.is_empty() {
        info!("GET /speedtest - Missing server_id query parameter");
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header(header::CONTENT_TYPE, "text/plain")
            .body(Body::from("Missing server_id query parameter"))
            .unwrap();
    }

    info!(
        "GET /speedtest - Starting speedtest for server_id: {}",
        params.server_id
    );

    match spawn_blocking(run_speedtest)
        .await
        .expect("Failed to spawn task")
    {
        Ok(result) => {
            info!(
                "GET /speedtest - Speedtest completed for server {} ({}) - Download: {:.2} Mbps, Upload: {:.2} Mbps, Ping: {:.2} ms",
                result.server.name,
                result.server.id,
                result.download.bandwidth as f64 / 125000.0,
                result.upload.bandwidth as f64 / 125000.0,
                result.ping.latency
            );

            let request_state = RequestState::new();
            request_state.set(&result);
            let body = request_state.encode();

            debug!("GET /speedtest - Encoded {} bytes of metrics", body.len());

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/plain; version=0.0.4")
                .body(Body::from(body))
                .unwrap()
        }
        Err(e) => {
            error!(
                "GET /speedtest - Speedtest failed for server_id {}: {}",
                params.server_id, e
            );
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(header::CONTENT_TYPE, "text/plain")
                .body(Body::from(format!("Speedtest failed: {}", e)))
                .unwrap()
        }
    }
}

async fn handle_metrics(State(app_state): State<Arc<AppState>>) -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    encoder
        .encode(&app_state.registry.gather(), &mut buffer)
        .unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from(buffer))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_speedtest_metrics() {
        let json_str =
            fs::read_to_string("tests/test_data.json").expect("Failed to read test data file");
        let result: SpeedtestResult =
            serde_json::from_str(&json_str).expect("Failed to parse test data");

        let request_state = RequestState::new();
        request_state.set(&result);
        let metrics_output = String::from_utf8(request_state.encode()).unwrap();

        let expected_metrics = [
            ("speedtest_ping_latency_seconds{isp=\"Test ISP\",server_id=\"52533\",server_name=\"Virtual Machines\"} 0.01228", "ping latency"),
            ("speedtest_download_bandwidth_bytes{isp=\"Test ISP\",server_id=\"52533\",server_name=\"Virtual Machines\"} 39924051", "download bandwidth"),
            ("speedtest_upload_bandwidth_bytes{isp=\"Test ISP\",server_id=\"52533\",server_name=\"Virtual Machines\"} 13008272", "upload bandwidth"),
        ];

        for (metric, description) in expected_metrics {
            assert!(
                metrics_output.contains(metric),
                "Missing or incorrect {} metric",
                description
            );
        }
    }

    #[tokio::test]
    async fn test_no_metric_leakage_between_requests() {
        let json1 =
            fs::read_to_string("tests/test_data.json").expect("Failed to read test data file");
        let json2 = fs::read_to_string("tests/test_data_2.json")
            .expect("Failed to read second test data file");

        let result1: SpeedtestResult =
            serde_json::from_str(&json1).expect("Failed to parse test data");
        let result2: SpeedtestResult =
            serde_json::from_str(&json2).expect("Failed to parse second test data");

        let request_state1 = RequestState::new();
        request_state1.set(&result1);
        let output1 = String::from_utf8(request_state1.encode()).unwrap();

        let request_state2 = RequestState::new();
        request_state2.set(&result2);
        let output2 = String::from_utf8(request_state2.encode()).unwrap();

        assert!(
            output1.contains("server_name=\"Virtual Machines\""),
            "First response should contain server 1 name"
        );
        assert!(
            !output1.contains("server_name=\"Another Server\""),
            "First response must not contain server 2 name"
        );

        assert!(
            output2.contains("server_name=\"Another Server\""),
            "Second response should contain server 2 name"
        );
        assert!(
            !output2.contains("server_name=\"Virtual Machines\""),
            "Second response must not contain server 1 name"
        );
    }
}

