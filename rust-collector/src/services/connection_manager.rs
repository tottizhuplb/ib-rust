use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::info;

use crate::ib::IbGatewayClient;
use crate::pipeline::MarketDataSource;

pub struct ConnectionManager {
    client: Arc<Mutex<IbGatewayClient>>,
}

impl ConnectionManager {
    pub fn new(client: Arc<Mutex<IbGatewayClient>>) -> Self {
        Self { client }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        {
            let mut client = self.client.lock().await;
            client.connect().await?;
        }

        info!("connection manager ready");
        Ok(())
    }
}
