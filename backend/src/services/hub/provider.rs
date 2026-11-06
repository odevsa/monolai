use crate::core::error::AppResult;
use crate::domain::{HubModelItem, HubModelReadme, RepositoryRecord};

#[derive(Debug, Clone, Default)]
pub struct RepositorySearchQuery<'a> {
    pub query: Option<&'a str>,
    pub modality: Option<&'a str>,
    pub sort: Option<&'a str>,
    pub page: Option<u32>,
}

pub trait ModelRepositoryProvider: Send + Sync {
    #[allow(dead_code)]
    fn repo_type(&self) -> &'static str;

    fn search<'a>(
        &'a self,
        repo: &'a RepositoryRecord,
        query: RepositorySearchQuery<'a>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<Vec<HubModelItem>>> + Send + 'a>>;

    fn get_model_details<'a>(
        &'a self,
        repo: &'a RepositoryRecord,
        model_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<HubModelItem>> + Send + 'a>>;

    fn get_model_readme<'a>(
        &'a self,
        repo: &'a RepositoryRecord,
        model_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<HubModelReadme>> + Send + 'a>>;
}
