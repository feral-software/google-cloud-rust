use std::env;

use thiserror::Error;

use google_cloud_gax::{
    conn::{ConnectionOptions, Environment},
    grpc::Status,
    retry::RetrySetting,
};
use google_cloud_token::NopeTokenSourceProvider;

use super::{task, Task};
use crate::apiv2;

#[derive(Debug, Clone)]
pub struct Client {
    location_id: String,
    project_id: String,
    inner: apiv2::Client,
}

impl Client {
    pub async fn default() -> Result<Self, Error> {
        Self::new(Configuration::default()).await
    }

    pub async fn new(config: Configuration) -> Result<Self, Error> {
        let pool_size = config.pool_size.unwrap_or_default();

        let inner = apiv2::Client::new(
            apiv2::ConnectionManager::new(
                pool_size,
                config.endpoint.as_str(),
                &config.environment,
                &ConnectionOptions::default(),
            )
            .await?,
        );

        Ok(Self {
            location_id: config.location_id,
            project_id: config.project_id.ok_or(Error::ProjectIdNotFound)?,
            inner,
        })
    }

    fn task(&self, queue_id: &str) -> Task {
        Task::new(self.fully_qualified_queue_name(queue_id), self.inner.clone())
    }

    pub async fn create_task(
        &self,
        id: Option<&str>,
        queue_id: &str,
        cfg: task::Config,
        retry: Option<RetrySetting>,
    ) -> Result<Task, Status> {
        let task = self.task(queue_id);
        let fqtn = id.map(|id| self.fully_qualified_task_name(queue_id, id));
        task.create(fqtn, cfg, retry).await.map(|_v| task)
    }

    pub fn fully_qualified_queue_name(&self, id: &str) -> String {
        if id.contains('/') {
            id.to_string()
        } else {
            format!("projects/{}/locations/{}/queues/{}", self.project_id, self.location_id, id)
        }
    }

    pub fn fully_qualified_task_name(&self, queue_id: &str, id: &str) -> String {
        if id.contains('/') {
            id.to_string()
        } else {
            format!(
                "projects/{}/locations/{}/queues/{}/tasks/{}",
                self.project_id, self.location_id, queue_id, id
            )
        }
    }
}

#[cfg(feature = "auth")]
pub use google_cloud_auth;

pub struct Configuration {
    pub pool_size: Option<usize>,
    // https://cloud.google.com/tasks/docs/reference/rpc/google.cloud.location
    pub location_id: String,
    pub project_id: Option<String>,
    pub environment: Environment,
    pub endpoint: String,
}

#[cfg(feature = "auth")]
impl Configuration {
    pub async fn with_auth(mut self) -> Result<Self, google_cloud_auth::error::Error> {
        if let Environment::GoogleCloud(_) = self.environment {
            let ts = google_cloud_auth::token::DefaultTokenSourceProvider::new(Self::auth_config()).await?;
            self.project_id = ts.project_id.clone();
            self.environment = Environment::GoogleCloud(Box::new(ts))
        }
        Ok(self)
    }

    fn auth_config() -> google_cloud_auth::project::Config<'static> {
        google_cloud_auth::project::Config {
            audience: Some(crate::apiv2::conn_pool::AUDIENCE),
            scopes: Some(&crate::apiv2::conn_pool::SCOPES),
            sub: None,
        }
    }
}

impl Default for Configuration {
    fn default() -> Self {
        let emulator = env::var("CLOUD_TASKS_EMULATOR_HOST").ok();
        let default_project_id = emulator.as_ref().map(|_| "local-project".to_string());
        Self {
            pool_size: Some(4),
            location_id: "us-east1".to_string(),
            project_id: default_project_id,
            environment: match emulator {
                Some(v) => Environment::Emulator(v),
                None => Environment::GoogleCloud(Box::new(NopeTokenSourceProvider {})),
            },
            endpoint: apiv2::CLOUD_TASKS.to_string(),
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    GAX(#[from] google_cloud_gax::conn::Error),
    #[error("invalid project_id")]
    ProjectIdNotFound,
}
