//! # Elastic Search utilities
//!
//! Module `elastic_search_utils` exports helper functions that facilitate storing Query Models in
//! Elastic Search.

use anyhow::Result;
use elasticsearch::Elasticsearch;
use elasticsearch::http::transport::Transport;
use log::{debug,warn};
use serde_json::Value;
use std::time;
use tokio::time::delay_for;
use elasticsearch::cluster::ClusterStatsParts;

/// Polls ElasticSearch until it is available and ready.
pub async fn wait_for_elastic_search() -> Result<Elasticsearch> {
    let interval = time::Duration::from_secs(1);
    loop {
        match try_to_connect().await {
            Err(e) => {
                warn!("Elastic Search is not ready (yet): {:?}", e);
            },
            Ok(client) => return Ok(client),
        }
        delay_for(interval).await;
    }
}

async fn try_to_connect() -> Result<Elasticsearch> {
    let transport = Transport::single_node("http://elastic-search:9200")?;
    let client = Elasticsearch::new(transport);
    let response = client
        .info()
        .send()
        .await?;

    let response_body = response.json::<Value>().await?;
    debug!("Info response body: {:?}", response_body);

    wait_for_status_ready(&client).await?;

    debug!("Elastic Search: contacted");

    Ok(client)
}

async fn wait_for_status_ready(client: &Elasticsearch) -> Result<()> {
    let interval = time::Duration::from_secs(1);
    loop {
        let response = client.cluster().stats(ClusterStatsParts::NodeId(&["*"])).send().await?;
        let response_body = response.json::<Value>().await?;
        let status = response_body.as_object().map(|o| o.get("status")).flatten();
        debug!("Elastic Search status: {:?}", status);
        if let Some(Value::String(status_code)) = status {
            if *status_code != "red" {
                return Ok(());
            }
        }
        delay_for(interval).await;
    }
}