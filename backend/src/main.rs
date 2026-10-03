mod config;
mod db;
mod docs;
mod handlers;
mod routes;
mod runtimes;
mod state;

use clap::Parser;
use config::load_config;
use runtimes::process_manager::{
    adopt_or_clean_orphans, start_idle_auto_unload_loop, ProcessManager,
};
use state::AppState;
use std::{
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to config YAML file (-c or --config)
    #[arg(short = 'c', long = "config")]
    config: Option<String>,

    /// Path to models folder
    #[arg(default_value = "~/models")]
    models_folder: String,

    /// Listen address (e.g. 0.0.0.0:8080)
    #[arg(short, long)]
    listen: Option<String>,

    /// Path to SQLite database
    #[arg(long)]
    db: Option<String>,

    /// Path to extra runtime manifests folder
    #[arg(long = "runtimes")]
    runtimes_folder: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "monolai=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    // Load config.yaml from CLI path or OS fallback path
    let (app_config, config_status) = load_config(cli.config.as_deref());

    if config_status.is_valid {
        tracing::info!(
            "Loaded config from: {}",
            config_status.loaded_path.as_deref().unwrap_or("unknown")
        );
    } else {
        tracing::warn!(
            "Config status invalid or missing. Expected path: {}. Error: {}",
            config_status.expected_path,
            config_status.error_message.as_deref().unwrap_or("none")
        );
    }

    let listen_addr_str = if let Some(l) = cli.listen {
        l
    } else {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        format!("{}:{}", host, port)
    };

    let addr: SocketAddr = listen_addr_str
        .parse()
        .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 8080)));

    let mut sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    );
    sys.refresh_all();

    let db_path = if let Some(ref d) = cli.db {
        config::expand_tilde(d).to_string_lossy().to_string()
    } else {
        config::get_default_db_path().to_string_lossy().to_string()
    };

    let db_pool = match db::init_db(&db_path).await {
        Ok(pool) => {
            tracing::info!("SQLite database initialized successfully at: {}", db_path);
            pool
        }
        Err(err) => {
            tracing::error!("Failed to initialize SQLite database at {}: {}", db_path, err);
            panic!("Database initialization failed: {}", err);
        }
    };

    let db_dir = std::path::Path::new(&db_path)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();

    let process_manager = ProcessManager::new(db_dir);

    // Adopt active processes from previous boot or clean up dead records
    adopt_or_clean_orphans(&process_manager).await;

    // Start background auto-unload monitoring loop for inactive models
    start_idle_auto_unload_loop(process_manager.clone(), db_pool.clone());



    let installer_manager = Arc::new(crate::runtimes::installer::RuntimeInstallerManager::new());

    let state = AppState {
        sys: Arc::new(Mutex::new(sys)),
        config: Arc::new(Mutex::new(app_config)),
        config_status: Arc::new(Mutex::new(config_status)),
        cli_config_path: cli.config.clone(),
        db: db_pool,
        process_manager,
        installer_manager,
    };

    let app = routes::create_router(state);

    let port = addr.port();
    tracing::info!("Server listening on http://{}", addr);
    println!("Monolai backend started:");
    println!("  - Local:      http://localhost:{}", port);
    println!("  - Swagger UI: http://localhost:{}/api/swagger", port);
    if let Some(ip) = get_local_network_ip() {
        println!("  - Network:    http://{}:{}", ip, port);
    }

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("Failed to bind to {}: {}", addr, e);
            eprintln!("\nFatal error: Failed to bind to port {} ({}).", addr.port(), e);
            eprintln!("Tip: Check if another process or Docker container is already using port {}.\n", addr.port());
            std::process::exit(1);
        }
    };
    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("Server error: {}", e);
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}

fn get_local_network_ip() -> Option<std::net::IpAddr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip())
}
