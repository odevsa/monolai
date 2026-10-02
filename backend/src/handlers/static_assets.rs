use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../frontend/build"]
pub struct Assets;

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() {
        return serve_asset("index.html");
    }

    if let Some(res) = serve_asset_opt(path) {
        return res;
    }

    serve_asset("index.html")
}

fn serve_asset_opt(path: &str) -> Option<Response> {
    let asset = Assets::get(path)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    Some(
        (
            [(header::CONTENT_TYPE, mime.as_ref())],
            Body::from(asset.data),
        )
            .into_response(),
    )
}

fn serve_asset(path: &str) -> Response {
    if let Some(res) = serve_asset_opt(path) {
        res
    } else {
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html")],
            Body::from("<!DOCTYPE html><html><head><title>Monolai</title></head><body><div id=\"app\">It's running.</div></body></html>"),
        )
            .into_response()
    }
}
