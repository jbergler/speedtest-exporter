# Speedtest Exporter

[![Build Status](https://github.com/jbergler/speedtest-exporter/workflows/build/badge.svg)](https://github.com/jbergler/speedtest-exporter/actions)
[![Version](https://img.shields.io/github/v/release/jbergler/speedtest-exporter.svg)](https://github.com/jbergler/speedtest-exporter/releases/latest)

A Prometheus exporter that runs speedtest.net measurements on-demand and exports the results as metrics.

**Note**: This is a fork of [lpicanco/prometheus-speedtest-exporter](https://github.com/lpicanco/prometheus-speedtest-exporter) that changes the behavior from periodic background testing to on-demand API calls.

## Features

- **On-demand speedtest API** - Call `GET /speedtest?server_id=<id>` to run a test and get results immediately
- Prometheus metrics for:
  - Ping latency (average, low, high)
  - Download performance (bandwidth, bytes, elapsed time, latency)
  - Upload performance (bandwidth, bytes, elapsed time, latency)
- Multi-architecture support (amd64, arm64, armv7)
- Docker support
- Minimal resource footprint

## Quick Start

Example Prometheus `scrape_config`:

```yaml
apiVersion: monitoring.coreos.com/v1alpha1
kind: ScrapeConfig
metadata:
  name: speedtest
spec:
  scrapeInterval: 1h
  scrapeTimeout: 3m
  metricsPath: /speedtest
  staticConfigs:
    - targets: # server id's from `speedtest -L`
        - "28463" # Chorus Fibre Lab
        - "17618" # Vocus Sydney
        - "37708" # Unlimited Fibre NYC
  relabelings:
    - action: replace
      sourceLabels: [__address__]
      targetLabel: __param_server_id
    - action: replace
      sourceLabels: [__param_server_id]
      targetLabel: instance
    - action: replace
      targetLabel: __address__
      replacement: speedtest-exporter.monitoring.svc.cluster.local:9516
  metricRelabelings:
    - action: labeldrop
      regex: (pod|instance)
```

## What's Different in This Fork?

This fork changes the architecture from periodic background testing to an on-demand model:

- Runs on demand only, periodic testing comes from your scrape config
- No metrics history, only serves fresh data
- More visibility via request logging

## Configuration

### Command-line options

```bash
speedtest-exporter [OPTIONS]

Options:
    --http-host <HOST>    Host to bind to [env: HTTP_HOST=] [default: 0.0.0.0]
    --http-port <PORT>    Port for metrics endpoint [env: HTTP_PORT=] [default: 9516]
    -h, --help            Print help
    -V, --version         Print version
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `HTTP_HOST` | Host to bind to | `0.0.0.0` |
| `HTTP_PORT` | Port for the metrics endpoint | `9516` |
| `RUST_LOG` | Log level (trace, debug, info, warn, error) | `info` |

### API Endpoints

#### GET /speedtest

Run a speedtest against a specific server.

**Query Parameters:**
- `server_id` (required): Speedtest server ID

**Example:**
```bash
curl "http://localhost:9516/speedtest?server_id=52533"
```

**Response:** Prometheus text format metrics

**Status Codes:**
- `200 OK` - Test successful
- `400 Bad Request` - Missing or empty server_id
- `500 Internal Server Error` - Speedtest execution failed

#### GET /metrics

Returns any global Prometheus metrics (if configured).

**Example:**
```bash
curl "http://localhost:9516/metrics"
```

## Metrics

All metrics include the following labels:
- `server_name`: Speedtest server name
- `server_id`: Speedtest server ID
- `isp`: Internet Service Provider name

### Available Metrics

```
# HELP speedtest_ping_latency_seconds Speedtest ping latency in seconds
# TYPE speedtest_ping_latency_seconds gauge
# HELP speedtest_ping_low_seconds Speedtest ping low in seconds
# TYPE speedtest_ping_low_seconds gauge
# HELP speedtest_ping_high_seconds Speedtest ping high in seconds
# TYPE speedtest_ping_high_seconds gauge

# HELP speedtest_download_bytes Number of bytes downloaded during speedtest
# TYPE speedtest_download_bytes gauge
# HELP speedtest_download_bandwidth_bytes Speedtest download bandwidth in bytes per second
# TYPE speedtest_download_bandwidth_bytes gauge
# HELP speedtest_download_elapsed_seconds Speedtest download elapsed time in seconds
# TYPE speedtest_download_elapsed_seconds gauge
# HELP speedtest_download_latency_iqm_seconds Speedtest download latency iqm in seconds
# TYPE speedtest_download_latency_iqm_seconds gauge
# HELP speedtest_download_latency_low_seconds Speedtest download latency low in seconds
# TYPE speedtest_download_latency_low_seconds gauge
# HELP speedtest_download_latency_high_seconds Speedtest download latency high in seconds
# TYPE speedtest_download_latency_high_seconds gauge

# HELP speedtest_upload_bytes Number of bytes uploaded during speedtest
# TYPE speedtest_upload_bytes gauge
# HELP speedtest_upload_bandwidth_bytes Speedtest upload bandwidth in bytes per second
# TYPE speedtest_upload_bandwidth_bytes gauge
# HELP speedtest_upload_elapsed_seconds Speedtest upload elapsed time in seconds
# TYPE speedtest_upload_elapsed_seconds gauge
# HELP speedtest_upload_latency_iqm_seconds Speedtest upload latency iqm in seconds
# TYPE speedtest_upload_latency_iqm_seconds gauge
# HELP speedtest_upload_latency_low_seconds Speedtest upload latency low in seconds
# TYPE speedtest_upload_latency_low_seconds gauge
# HELP speedtest_upload_latency_high_seconds Speedtest upload latency high in seconds
# TYPE speedtest_upload_latency_high_seconds gauge
```

## Development

### Running the exporter

```bash
# Start the server (listens on port 9516)
cargo run

# Or with custom host/port
cargo run -- --http-host 127.0.0.1 --http-port 8080
```

### Making speedtest requests

```bash
# Run a speedtest against a specific server and get Prometheus metrics
curl "http://localhost:9516/speedtest?server_id=52533"

# Response includes metrics:
# speedtest_ping_latency_seconds{...} 0.01228
# speedtest_download_bandwidth_bytes{...} 39924051
# speedtest_upload_bandwidth_bytes{...} 13008272
# ... and more
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Run tests and linting:
   ```bash
   cargo test
   cargo clippy
   ```
4. Commit your changes (`git commit -m 'Add some amazing feature'`)
5. Push to the branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request

## Security

Please report security issues via GitHub security advisories.

## License

This project is licensed under the Apache License - see the [LICENSE](LICENSE) file for details.

## Credits

- Original project: [lpicanco/prometheus-speedtest-exporter](https://github.com/lpicanco/prometheus-speedtest-exporter)
