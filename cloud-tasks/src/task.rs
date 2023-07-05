use std::time::SystemTime;

use google_cloud_gax::{grpc::Status, retry::RetrySetting};
use google_cloud_googleapis::cloud::tasks::v2 as google;

use crate::apiv2;

#[derive(Default)]
pub struct Config {
    pub schedule_time: Option<SystemTime>,
    pub http_request: google::HttpRequest,
}

#[derive(Debug, Clone)]
pub struct Task {
    fqqn: String,
    client: apiv2::Client,
}

impl Task {
    pub fn new(fqqn: String, client: apiv2::Client) -> Self {
        Self { fqqn, client }
    }

    pub async fn create(&self, fqtn: Option<String>, cfg: Config, retry: Option<RetrySetting>) -> Result<(), Status> {
        self.client
            .create_task(
                google::CreateTaskRequest {
                    parent: self.fqqn.clone(),
                    task: fqtn.map(|fqtn| google::Task {
                        name: fqtn.to_string(),
                        schedule_time: cfg.schedule_time.map(SystemTime::into),
                        create_time: None,
                        dispatch_deadline: None,
                        // output only?
                        dispatch_count: 0,
                        response_count: 0,
                        first_attempt: None,
                        last_attempt: None,
                        view: 0,
                        message_type: Some(google::task::MessageType::HttpRequest(cfg.http_request)),
                    }),
                    response_view: 0,
                },
                retry,
            )
            .await
            .map(|_v| ())
    }
}
