use clap::Parser;

#[derive(Parser, Debug, Clone, Default)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Host to bind to (can also be set via HTTP_HOST)
    #[arg(long, env = "HTTP_HOST", default_value = "0.0.0.0")]
    pub http_host: String,

    /// Port for Prometheus metrics endpoint (can also be set via HTTP_PORT)
    #[arg(long, env = "HTTP_PORT", default_value_t = 9516)]
    pub http_port: u16,
}
