mod core;
mod domain;
mod infrastructure;
mod services;
mod docs;
mod handlers;
mod routes;
mod state;

use clap::Parser;
use core::config::{expand_tilde, get_default_config_path, get_default_db_path, load_config, PathResolver};
use infrastructure::db::{init_db, ChatRepository, ModelRepository, SettingRepository};
use infrastructure::downloader::RuntimeInstallerManager;
use infrastructure::hardware::GpuTracker;
use infrastructure::process::manager::{
    adopt_or_clean_orphans, start_idle_auto_unload_loop, ProcessManager,
};
use services::{ChatService, HostService, ModelService, ProxyService, RuntimeService};
use state::AppState;
use std::{
    env,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};
use tokio::sync::RwLock;
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

    // 1. Load config.yaml from CLI path or OS fallback path
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

    // 2. Initialize PathResolver
    let path_resolver = PathResolver::new(
        &app_config,
        cli.config.as_deref(),
        Some(&cli.models_folder),
        cli.runtimes_folder.as_deref(),
        cli.db.as_deref(),
    );

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

    let db_path_str = path_resolver.db_path.to_string_lossy().to_string();

    // 3. Initialize SQLite DB pool
    let db_pool = match init_db(&db_path_str).await {
        Ok(pool) => {
            tracing::info!("SQLite database initialized successfully at: {}", db_path_str);
            pool
        }
        Err(err) => {
            tracing::error!("Failed to initialize SQLite database at {}: {}", db_path_str, err);
            panic!("Database initialization failed: {}", err);
        }
    };

    let db_dir = path_resolver
        .db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    // 4. Initialize Repositories
    let model_repo = ModelRepository::new(db_pool.clone());
    let chat_repo = ChatRepository::new(db_pool.clone());
    let setting_repo = SettingRepository::new(db_pool.clone()).await;

    // 5. Initialize Infrastructure Components
    let process_manager = ProcessManager::new(db_dir);
    let installer_manager = Arc::new(RuntimeInstallerManager::new());
    let gpu_tracker = Arc::new(GpuTracker::new());

    if let Some(ref gpu) = gpu_tracker.primary_gpu() {
        tracing::info!("Detected primary GPU: {} (vendor: {}, dedicated: {})", gpu.name, gpu.vendor, gpu.is_dedicated);
    } else {
        tracing::info!("No dedicated GPU detected on host system.");
    }

    // 6. Background tasks
    adopt_or_clean_orphans(&process_manager).await;
    start_idle_auto_unload_loop(process_manager.clone(), setting_repo.clone());

    // 7. Initialize Application Services
    let config_lock = Arc::new(RwLock::new(app_config));
    let config_status_lock = Arc::new(RwLock::new(config_status));

    let model_service = Arc::new(ModelService::new(
        model_repo.clone(),
        setting_repo.clone(),
        process_manager.clone(),
        config_lock.clone(),
    ));

    let chat_service = Arc::new(ChatService::new(chat_repo.clone()));

    let extra_manifests_path = cli.runtimes_folder.as_ref().map(PathBuf::from);
    let runtime_service = Arc::new(RuntimeService::new(
        installer_manager,
        config_lock.clone(),
        extra_manifests_path,
    ));

    let host_service = Arc::new(HostService::new(
        Arc::new(Mutex::new(sys)),
        gpu_tracker,
    ));

    let proxy_service = Arc::new(ProxyService::new(
        model_repo,
        setting_repo.clone(),
        process_manager,
        config_lock.clone(),
    ));

    // 8. Build AppState
    let state = AppState {
        db: db_pool,
        config: config_lock,
        config_status: config_status_lock,
        path_resolver: Arc::new(path_resolver),
        cli_config_path: cli.config.clone(),
        model_service,
        chat_service,
        runtime_service,
        host_service,
        proxy_service,
        setting_repo,
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
        expand_tilde(d).to_string_lossy().to_string()
    } else {
        get_default_db_path().to_string_lossy().to_string()
    };
    let db_path = Path::new(&db_path_str);
    let mut removed_count = 0;

    let files_to_remove = [
        db_path.to_path_buf(),
        PathBuf::from(format!("{}-wal", db_path_str)),
        PathBuf::from(format!("{}-shm", db_path_str)),
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

    let db_dir = db_path.parent().unwrap_or_else(|| Path::new("."));
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
    handle_clean_storage(db_arg);

    let mut removed_config_count = 0;
    let mut config_paths = Vec::new();

    if let Some(c) = config_arg {
        config_paths.push(expand_tilde(c));
    }
    config_paths.push(get_default_config_path());
    config_paths.push(PathBuf::from("config.yaml"));

    let (_, status) = load_config(config_arg);
    if let Some(ref p) = status.loaded_path {
        config_paths.push(PathBuf::from(p));
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
