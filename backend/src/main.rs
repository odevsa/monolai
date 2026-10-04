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

    /// Host address to bind
    #[arg(long)]
    host: Option<String>,

    /// Port to bind
    #[arg(short = 'p', long)]
    port: Option<u16>,

    /// Path to SQLite database
    #[arg(long)]
    db: Option<String>,

    /// Path to extra runtime manifests folder
    #[arg(long = "runtimes")]
    runtimes_folder: Option<String>,

    /// Clean storage: delete SQLite database and active process cache
    #[arg(long = "clean-storage")]
    clean_storage: bool,

    /// Factory reset: delete database, process cache, and configuration
    #[arg(long = "factory-reset")]
    factory_reset: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.factory_reset {
        handle_factory_reset(cli.db.as_deref(), cli.config.as_deref());
        std::process::exit(0);
    } else if cli.clean_storage {
        handle_clean_storage(cli.db.as_deref());
        std::process::exit(0);
    }

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "monolai=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

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
        let host = cli
            .host
            .or_else(|| env::var("HOST").ok())
            .or_else(|| app_config.host.clone())
            .unwrap_or_else(|| "0.0.0.0".to_string());
        let port = cli
            .port
            .or_else(|| env::var("PORT").ok().and_then(|p| p.parse().ok()))
            .or(app_config.port)
            .unwrap_or(8080);
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
    let gpu_tracker = Arc::new(crate::runtimes::hardware::GpuTracker::new());

    if let Some(ref gpu) = gpu_tracker.primary_gpu() {
        tracing::info!("Detected primary GPU: {} (vendor: {}, dedicated: {})", gpu.name, gpu.vendor, gpu.is_dedicated);
    } else {
        tracing::info!("No dedicated GPU detected on host system.");
    }

    let state = AppState {
        sys: Arc::new(Mutex::new(sys)),
        config: Arc::new(Mutex::new(app_config)),
        config_status: Arc::new(Mutex::new(config_status)),
        cli_config_path: cli.config.clone(),
        db: db_pool,
        process_manager,
        installer_manager,
        gpu_tracker,
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

fn handle_clean_storage(db_arg: Option<&str>) {
    let db_path_str = if let Some(d) = db_arg {
        config::expand_tilde(d).to_string_lossy().to_string()
    } else {
        config::get_default_db_path().to_string_lossy().to_string()
    };
    let db_path = std::path::Path::new(&db_path_str);
    let mut removed_count = 0;

    let files_to_remove = [
        db_path.to_path_buf(),
        std::path::PathBuf::from(format!("{}-wal", db_path_str)),
        std::path::PathBuf::from(format!("{}-shm", db_path_str)),
    ];

    for file in &files_to_remove {
        if file.exists() {
            match std::fs::remove_file(file) {
                Ok(_) => {
                    println!("Removed database file: {}", file.display());
                    removed_count += 1;
                }
                Err(e) => eprintln!("Failed to remove {}: {}", file.display(), e),
            }
        }
    }

    let db_dir = db_path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let proc_json = db_dir.join("processes.json");
    if proc_json.exists() {
        match std::fs::remove_file(&proc_json) {
            Ok(_) => {
                println!("Removed process cache: {}", proc_json.display());
                removed_count += 1;
            }
            Err(e) => eprintln!("Failed to remove {}: {}", proc_json.display(), e),
        }
    }

    println!(
        "Clean Storage completed successfully ({} file(s) removed).",
        removed_count
    );
}

fn handle_factory_reset(db_arg: Option<&str>, config_arg: Option<&str>) {
    // 1. Clean storage first
    handle_clean_storage(db_arg);

    // 2. Remove configuration files
    let mut removed_config_count = 0;
    let mut config_paths = Vec::new();

    if let Some(c) = config_arg {
        config_paths.push(config::expand_tilde(c));
    }
    config_paths.push(config::get_default_config_path());
    config_paths.push(std::path::PathBuf::from("config.yaml"));

    let (_, status) = config::load_config(config_arg);
    if let Some(ref p) = status.loaded_path {
        config_paths.push(std::path::PathBuf::from(p));
    }

    config_paths.sort();
    config_paths.dedup();

    for path in config_paths {
        if path.exists() {
            match std::fs::remove_file(&path) {
                Ok(_) => {
                    println!("Removed config file: {}", path.display());
                    removed_config_count += 1;
                }
                Err(e) => eprintln!("Failed to remove config {}: {}", path.display(), e),
            }
        }
    }

    println!(
        "Factory Reset completed successfully ({} config file(s) removed).",
        removed_config_count
    );
}
