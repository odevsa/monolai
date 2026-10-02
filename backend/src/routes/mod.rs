use axum::{
    response::Redirect,
    routing::{any, get, post, put},
    Router,
};
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
        health::health_handler,
        host::{sysinfo_handler, sysinfo_stream_handler},
        models::{
            available_models_handler, create_model_handler, delete_model_handler,
            get_models_handler, get_running_models_handler, load_model_handler, swap_model_handler,
            unload_all_models_handler, unload_model_handler, update_model_handler,
        },
        openai_proxy::{v1_model_by_id_handler, v1_models_handler, v1_proxy_handler},
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

/// Builds the complete application router with organized sub-routes, Swagger UI, and static fallback.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check alias at root (for Docker healthcheck, load balancers, etc.)
        .route("/health", get(health_handler))
        .nest("/api", api_router())
        .nest("/v1", openai_router())
        .merge(swagger_router())
        .fallback(static_handler)
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
        // Config & Setup
        .route("/config/status", get(config_status_handler))
        .route("/config/setup", post(save_setup_config_handler))
        // Runtimes & Manifests
        .route("/runtimes", get(runtimes_handler))
        .route("/runtimes/:id/install", post(install_runtime_handler))
        .route("/runtimes/:id/install/stream", get(install_stream_handler))
        .route("/runtimes/:id", axum::routing::delete(uninstall_runtime_handler))
        .route("/runtime-manifests", get(get_runtime_manifests_handler))
        .route(
            "/runtime-manifests/:id",
            get(get_runtime_manifest_by_id_handler),
        )
        // Models
        .route("/models/available", get(available_models_handler))
        .route(
            "/models",
            get(get_models_handler).post(create_model_handler),
        )
        .route("/models/unload-all", post(unload_all_models_handler))
        .route(
            "/models/:id",
            put(update_model_handler).delete(delete_model_handler),
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
}

/// OpenAI Compatibility proxy routes (/v1/*)
fn openai_router() -> Router<AppState> {
    Router::new()
        .route("/models", get(v1_models_handler))
        .route("/models/:id", get(v1_model_by_id_handler))
        .route("/*path", any(v1_proxy_handler))
}

/// Swagger UI and OpenAPI documentation routes
fn swagger_router() -> Router<AppState> {
    Router::new()
        .merge(SwaggerUi::new("/api/swagger").url("/api/openapi.json", ApiDoc::openapi()))
        .route("/api/docs", get(|| async { Redirect::permanent("/api/swagger/") }))
        .route("/swagger", get(|| async { Redirect::permanent("/api/swagger/") }))
        .route("/docs", get(|| async { Redirect::permanent("/api/swagger/") }))
}
