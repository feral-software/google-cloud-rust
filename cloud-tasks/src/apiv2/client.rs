use std::sync::Arc;

mod gax {
    pub use google_cloud_gax::{
        conn::Channel,
        create_request, grpc,
        retry::{self, RetrySetting},
    };
}

mod google {
    pub use google_cloud_googleapis::cloud::tasks::v2::{
        cloud_tasks_client::CloudTasksClient, CreateTaskRequest, Task,
    };
}

use super::conn_pool::ConnectionManager;

#[derive(Clone, Debug)]
pub struct Client {
    cm: Arc<ConnectionManager>,
}

impl Client {
    pub fn new(cm: ConnectionManager) -> Self {
        Self { cm: Arc::new(cm) }
    }

    fn client(&self) -> google::CloudTasksClient<gax::Channel> {
        google::CloudTasksClient::new(self.cm.conn())
    }

    pub async fn create_task(
        &self,
        req: google::CreateTaskRequest,
        retry: Option<gax::RetrySetting>,
    ) -> Result<gax::grpc::Response<google::Task>, gax::grpc::Status> {
        let parent: &String = &req.parent;
        let action = || async {
            let mut client = self.client();
            let request = gax::create_request(format!("parent={parent}"), req.clone());
            client.create_task(request).await
        };
        gax::retry::invoke(retry, action).await
    }
}
