use crate::core::error::AppResult;
use crate::domain::{
    CreateRepositoryPayload, HubModelItem, RepositoryRecord, UpdateRepositoryPayload,
};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct SearchHubQuery {
    pub repo_id: Option<String>,
    pub query: Option<String>,
    pub modality: Option<String>,
    pub sort: Option<String>,
    pub page: Option<u32>,
}

#[utoipa::path(
    get,
    path = "/api/hub/repositories",
    tag = "Hub",
    responses(
        (status = 200, description = "List of configured Hub repositories", body = Vec<RepositoryRecord>)
    )
)]
pub async fn get_repositories_handler(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<RepositoryRecord>>> {
    let repos = state.hub_service.get_repositories().await?;
    Ok(Json(repos))
}

#[utoipa::path(
    post,
    path = "/api/hub/repositories",
    tag = "Hub",
    request_body = CreateRepositoryPayload,
    responses(
        (status = 201, description = "Repository registered successfully", body = RepositoryRecord)
    )
)]
pub async fn create_repository_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateRepositoryPayload>,
) -> AppResult<(StatusCode, Json<RepositoryRecord>)> {
    let repo = state.hub_service.create_repository(payload).await?;
    Ok((StatusCode::CREATED, Json(repo)))
}

#[utoipa::path(
    put,
    path = "/api/hub/repositories/{id}",
    tag = "Hub",
    params(
        ("id" = String, Path, description = "Repository ID")
    ),
    request_body = UpdateRepositoryPayload,
    responses(
        (status = 200, description = "Repository updated successfully", body = RepositoryRecord)
    )
)]
pub async fn update_repository_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateRepositoryPayload>,
) -> AppResult<Json<RepositoryRecord>> {
    let repo = state.hub_service.update_repository(&id, payload).await?;
    Ok(Json(repo))
}

#[utoipa::path(
    delete,
    path = "/api/hub/repositories/{id}",
    tag = "Hub",
    params(
        ("id" = String, Path, description = "Repository ID")
    ),
    responses(
        (status = 204, description = "Repository deleted successfully")
    )
)]
pub async fn delete_repository_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    state.hub_service.delete_repository(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/hub/models",
    tag = "Hub",
    params(
        ("repo_id" = Option<String>, Query, description = "Specific repository to search in"),
        ("query" = Option<String>, Query, description = "Search keywords"),
        ("modality" = Option<String>, Query, description = "Modality filter (text-to-text, text-to-image, etc)"),
        ("sort" = Option<String>, Query, description = "Sort criteria: downloads, likes, recent")
    ),
    responses(
        (status = 200, description = "Search results from repository Hub", body = Vec<HubModelItem>)
    )
)]
pub async fn search_hub_models_handler(
    State(state): State<AppState>,
    Query(query): Query<SearchHubQuery>,
) -> AppResult<Json<Vec<HubModelItem>>> {
    let models = state
        .hub_service
        .search_models(
            query.repo_id.as_deref(),
            query.query.as_deref(),
            query.modality.as_deref(),
            query.sort.as_deref(),
            query.page,
        )
        .await?;
    Ok(Json(models))
}

#[utoipa::path(
    get,
    path = "/api/hub/models/{repo_id}/{model_id}",
    tag = "Hub",
    params(
        ("repo_id" = String, Path, description = "Repository ID"),
        ("model_id" = String, Path, description = "Model ID on the hub")
    ),
    responses(
        (status = 200, description = "Model details with files and quants", body = HubModelItem)
    )
)]
pub async fn get_hub_model_details_handler(
    State(state): State<AppState>,
    Path((repo_id, model_id)): Path<(String, String)>,
) -> AppResult<Json<HubModelItem>> {
    let details = state
        .hub_service
        .get_model_details(&repo_id, &model_id)
        .await?;
    Ok(Json(details))
}
