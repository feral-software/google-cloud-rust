use google_cloud_gax::conn as gax;

pub const AUDIENCE: &str = "https://cloudtasks.googleapis.com/";
pub const SCOPES: [&str; 1] = ["https://www.googleapis.com/auth/cloud-platform"];

#[derive(Debug)]
pub struct ConnectionManager {
    inner: gax::ConnectionManager,
}

impl ConnectionManager {
    pub async fn new(
        pool_size: usize,
        domain: &str,
        environment: &gax::Environment,
        conn_options: &gax::ConnectionOptions,
    ) -> Result<Self, gax::Error> {
        Ok(ConnectionManager {
            inner: gax::ConnectionManager::new(pool_size, domain, AUDIENCE, environment, conn_options).await?,
        })
    }

    pub fn conn(&self) -> gax::Channel {
        self.inner.conn()
    }
}
