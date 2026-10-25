use axum::{
    body::Body,
    extract::ConnectInfo,
    http::Request,
    middleware::{self, Next},
    response::{Redirect, Response},
    routing::{any, get, post, put},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    docs::ApiDoc,
    handlers::{
        chats::{
            clear_chat_messages_handler, create_chat_handler, create_chat_message_handler,
            delete_chat_handler, delete_chat_message_handler, get_chat_messages_handler,
            get_chats_handler, update_chat_handler, update_chat_message_handler,
        },
        config::{config_status_handler, hardware_detect_handler, save_setup_config_handler},
        downloads::{
            cancel_download_handler, delete_download_handler, downloads_stream_handler,
            enqueue_download_handler, get_downloads_handler, pause_download_handler,
            resume_download_handler,
        },
        health::health_handler,
        host::{sysinfo_handler, sysinfo_stream_handler},
        hub::{
            create_repository_handler, delete_repository_handler, get_hub_model_details_handler,
            get_repositories_handler, search_hub_models_handler, update_repository_handler,
        },
        logs::{
            clear_logs_handler, download_logs_handler, get_log_services_handler, get_logs_handler,
            logs_stream_handler,
        },
        models::{
            available_models_handler, create_model_preset_handler, delete_model_handler,
            delete_model_preset_handler, get_compatible_runtimes_handler, get_model_by_id_handler,
            get_model_presets_handler, get_models_handler, get_running_models_handler,
            load_model_handler, running_models_stream_handler, set_default_model_preset_handler,
            swap_model_handler, unload_all_models_handler, unload_model_handler,
            update_model_preset_handler,
        },
        openai_proxy::{
            sdcpp_proxy_handler, v1_model_by_id_handler, v1_models_handler, v1_proxy_handler,
        },
        runtime_manifests::{get_runtime_manifest_by_id_handler, get_runtime_manifests_handler},
        runtimes::{
            install_runtime_handler, install_stream_handler, runtimes_handler,
            uninstall_runtime_handler,
        },
        settings::{
            delete_setting_handler, get_all_settings_handler, get_setting_handler,
            update_settings_handler,
        },
        static_assets::static_handler,
    },
    state::AppState,
};

async fn request_logging_middleware(req: Request<Body>, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path().to_string();

    let client_ip = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .or_else(|| {
            req.headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        })
        .or_else(|| {
            req.headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "127.0.0.1".to_string());

    let start = std::time::Instant::now();
    let response = next.run(req).await;
    let latency = start.elapsed();
    let status = response.status().as_u16();

    let is_telemetry_stream = path.contains("/stream") || path.contains("/host/usage");
    let is_static_asset = path.starts_with("/_app/")
        || path.ends_with(".js")
        || path.ends_with(".css")
        || path.ends_with(".svg")
        || path.ends_with(".ico");

    if !is_telemetry_stream && !is_static_asset {
        if status >= 500 {
            tracing::error!(
                target: "monolai",
                "[{}] {} {} -> {} ({:?})",
                client_ip, method, path, status, latency
            );
        } else if status >= 400 {
            tracing::warn!(
                target: "monolai",
                "[{}] {} {} -> {} ({:?})",
                client_ip, method, path, status, latency
            );
        } else {
            tracing::info!(
                target: "monolai",
                "[{}] {} {} -> {} ({:?})",
                client_ip, method, path, status, latency
            );
        }
    }

    response
}

/// Builds the complete application router with organized sub-routes, Swagger UI, and static fallback.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check alias at root (for Docker healthcheck, load balancers, etc.)
        .route("/health", get(health_handler))
        .nest("/api", api_router())
        .nest("/v1", openai_router())
        .nest("/sdcpp", sdcpp_router())
        .merge(swagger_router())
        .fallback(static_handler)
        .layer(middleware::from_fn(request_logging_middleware))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Core REST API routes (/api/*)
fn api_router() -> Router<AppState> {
    Router::new()
        // Health check endpoint (/api/health)
        .route("/health", get(health_handler))
        // Host & System Information
        .route("/host", get(sysinfo_handler))
        .route("/host/usage", get(sysinfo_stream_handler))
        .route("/hardware/detect", get(hardware_detect_handler))
        .route("/state", get(get_running_models_handler))
        .route("/state/stream", get(running_models_stream_handler))
        // Config & Setup
        .route("/config/status", get(config_status_handler))
        .route("/config/setup", post(save_setup_config_handler))
        // Runtimes & Manifests
        .route("/runtimes", get(runtimes_handler))
        .route("/runtimes/:id/install", post(install_runtime_handler))
        .route("/runtimes/:id/install/stream", get(install_stream_handler))
        .route(
            "/runtimes/:id",
            axum::routing::delete(uninstall_runtime_handler),
        )
        .route("/runtimes/compatible", get(get_compatible_runtimes_handler))
        .route("/runtime-manifests", get(get_runtime_manifests_handler))
        .route(
            "/runtime-manifests/:id",
            get(get_runtime_manifest_by_id_handler),
        )
        // Hub Repositories & Search
        .route(
            "/hub/repositories",
            get(get_repositories_handler).post(create_repository_handler),
        )
        .route(
            "/hub/repositories/:id",
            put(update_repository_handler).delete(delete_repository_handler),
        )
        .route("/hub/models", get(search_hub_models_handler))
        .route("/hub/models/:repo_id/*model_id", get(get_hub_model_details_handler))
        // Background Downloads
        .route(
            "/downloads",
            get(get_downloads_handler).post(enqueue_download_handler),
        )
        .route("/downloads/:id/pause", post(pause_download_handler))
        .route("/downloads/:id/resume", post(resume_download_handler))
        .route("/downloads/:id/cancel", post(cancel_download_handler))
        .route("/downloads/:id", axum::routing::delete(delete_download_handler))
        .route("/downloads/stream", get(downloads_stream_handler))
        // Models & Presets
        .route("/models/available", get(available_models_handler))
        .route("/models", get(get_models_handler))
        .route("/models/unload-all", post(unload_all_models_handler))
        .route(
            "/models/:id",
            get(get_model_by_id_handler).delete(delete_model_handler),
        )
        .route(
            "/models/:id/presets",
            get(get_model_presets_handler).post(create_model_preset_handler),
        )
        .route(
            "/models/presets/:preset_id",
            put(update_model_preset_handler).delete(delete_model_preset_handler),
        )
        .route(
            "/models/:id/presets/:preset_id/set-default",
            post(set_default_model_preset_handler),
        )
        .route("/models/:id/load", post(load_model_handler))
        .route("/models/:id/swap", post(swap_model_handler))
        .route("/models/:id/unload", post(unload_model_handler))
        // Settings
        .route(
            "/settings",
            get(get_all_settings_handler).post(update_settings_handler),
        )
        .route(
            "/settings/:key",
            get(get_setting_handler).delete(delete_setting_handler),
        )
        // Chats
        .route("/chats", get(get_chats_handler).post(create_chat_handler))
        .route(
            "/chats/:id",
            put(update_chat_handler).delete(delete_chat_handler),
        )
        .route(
            "/chats/:id/messages",
            get(get_chat_messages_handler)
                .post(create_chat_message_handler)
                .delete(clear_chat_messages_handler),
        )
        .route(
            "/chats/:id/messages/:msg_id",
            put(update_chat_message_handler).delete(delete_chat_message_handler),
        )
        // Logs
        .route(
            "/logs",
            get(get_logs_handler).delete(clear_logs_handler),
        )
        .route("/logs/services", get(get_log_services_handler))
        .route("/logs/stream", get(logs_stream_handler))
        .route("/logs/download", get(download_logs_handler))
}

/// OpenAI Compatibility proxy routes (/v1/*)
fn openai_router() -> Router<AppState> {
    Router::new()
        .route("/models", get(v1_models_handler))
        .route("/models/:id", get(v1_model_by_id_handler))
        .route("/*path", any(v1_proxy_handler))
}

/// Stable Diffusion C++ proxy routes (/sdcpp/*)
fn sdcpp_router() -> Router<AppState> {
    Router::new().route("/*path", any(sdcpp_proxy_handler))
}

/// Swagger UI and OpenAPI documentation routes
fn swagger_router() -> Router<AppState> {
    Router::new()
        .merge(SwaggerUi::new("/api/swagger").url("/api/openapi.json", ApiDoc::openapi()))
        .route(
            "/api/docs",
            get(|| async { Redirect::permanent("/api/swagger/") }),
        )
        .route(
            "/swagger",
            get(|| async { Redirect::permanent("/api/swagger/") }),
        )
        .route(
            "/docs",
            get(|| async { Redirect::permanent("/api/swagger/") }),
        )
}
