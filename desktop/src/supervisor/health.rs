use std::net::TcpListener;
use std::time::Duration;
use crate::core::config::GuiConfig;

/// Verifies whether the specified host and port are available for binding.
pub fn check_port_available(host: &str, port: u16) -> Result<(), String> {
    let test = TcpListener::bind((host, port))
        .or_else(|_| TcpListener::bind(("127.0.0.1", port)));

    match test {
        Ok(listener) => {
            drop(listener);
            Ok(())
        }
        Err(e) => Err(format!(
            "Port {} is already in use ({}). Check if Docker or another instance is running!",
            port, e
        )),
    }
}

/// Probes the HTTP health check endpoint (`/health`) on 127.0.0.1 with localhost fallback.
pub fn probe_health(port: u16, timeout: Duration) -> bool {
    let health_url = format!("http://127.0.0.1:{}/health", port);
    let resp = ureq::get(&health_url)
        .timeout(timeout)
        .call()
        .or_else(|_| {
            let fallback_url = format!("http://localhost:{}/health", port);
            ureq::get(&fallback_url).timeout(timeout).call()
        });

    matches!(resp, Ok(r) if r.status() == 200)
}

/// Dispatches a configuration update notification to the running backend via HTTP API.
pub fn notify_backend_config(port: u16, config: &GuiConfig) -> Result<(), String> {
    let payload = serde_json::json!({
        "models": config.models_dir.trim(),
        "runtimes": config.runtimes_dir.trim(),
        "hardware": config.hardware.trim(),
        "host": config.host.trim(),
        "port": config.port,
    });

    let body = serde_json::to_string(&payload)
        .map_err(|e| format!("Failed to serialize config payload: {}", e))?;

    let url = format!("http://127.0.0.1:{}/api/config/setup", port);
    ureq::post(&url)
        .set("Content-Type", "application/json")
        .timeout(Duration::from_millis(800))
        .send_string(&body)
        .map_err(|e| format!("Backend HTTP notification error: {}", e))?;

    Ok(())
}
