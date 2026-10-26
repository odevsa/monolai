use std::net::TcpListener;
use std::time::Duration;

/// Verifies whether the specified host and port are available for binding.
pub fn check_port_available(host: &str, port: u16) -> Result<(), String> {
    let test = TcpListener::bind((host, port)).or_else(|_| TcpListener::bind(("127.0.0.1", port)));

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
    let resp = ureq::get(&health_url).timeout(timeout).call().or_else(|_| {
        let fallback_url = format!("http://localhost:{}/health", port);
        ureq::get(&fallback_url).timeout(timeout).call()
    });

    matches!(resp, Ok(r) if r.status() == 200)
}
