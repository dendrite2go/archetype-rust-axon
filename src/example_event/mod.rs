use anyhow::{Context,Result};
use dendrite::axon_utils::{AsyncApplicableTo, AxonServerHandle, TheHandlerRegistry, TokenStore, event_processor, empty_handler_registry};
use elasticsearch::{Elasticsearch, IndexParts, GetParts};
use log::{debug,error};
use prost::Message;
use serde_json::{json, Value};
use sha2::{Sha256, Digest};
use crate::elastic_search_utils::wait_for_elastic_search;
use crate::grpc_example::{GreetedEvent,Greeting};

#[derive(Clone)]
struct ExampleQueryModel {
    es_client: Elasticsearch,
}

#[tonic::async_trait]
impl TokenStore for ExampleQueryModel {
    async fn store_token(&self, token: i64) {
        let hex_token = format!("{:x}", token);
        let result = self.es_client
            .index(IndexParts::IndexId("tracking-token", "greeting"))
            .body(json!({
                    "id": "greeting",
                    "token": hex_token,
                }))
            .send()
            .await
        ;
        debug!("Elastic Search store token result: {:?}", result);
    }

    async fn retrieve_token(&self) -> Result<i64> {
        let response = self.es_client
            .get(GetParts::IndexId("tracking-token", "greeting"))
            ._source(&["token"])
            .send()
            .await?
        ;
        let value = response.json::<Value>().await?;
        debug!("Retrieved response value: {:?}", value);
        if let Value::String(hex_token) = &value["_source"]["token"] {
            let token = i64::from_str_radix(hex_token, 16)?;
            debug!("Retrieved token: {:?}", token);
            return Ok(token);
        }
        Ok(-1)
    }
}

/// Handles events for the example application.
///
/// Constructs an event handler registry and delegates to function `event_processor`.
pub async fn process_events(axon_server_handle : AxonServerHandle) {
    if let Err(e) = internal_process_events(axon_server_handle).await {
        error!("Error while handling commands: {:?}", e);
    }
    debug!("Stopped handling commands for example application");
}

async fn internal_process_events(axon_server_handle : AxonServerHandle) -> Result<()> {
    let client = wait_for_elastic_search().await?;
    debug!("Elastic Search client: {:?}", client);

    let query_model = ExampleQueryModel {
        es_client: client,
    };

    let mut event_handler_registry: TheHandlerRegistry<ExampleQueryModel,Option<ExampleQueryModel>> = empty_handler_registry();

    event_handler_registry.register(&handle_greeted_event)?;

    event_processor(axon_server_handle, query_model, event_handler_registry).await.context("Error while handling commands")
}

#[dendrite_macros::event_handler]
pub async fn handle_greeted_event(event: GreetedEvent, query_model: ExampleQueryModel) -> Result<()> {
    debug!("Apply greeted event to ExampleQueryModel");
    let es_client = query_model.es_client.clone();
    if let Some(Greeting {message}) = event.message.clone() {
        let value = message.clone();
        let mut hasher = Sha256::new();
        Digest::update(&mut hasher,&message);
        let hash: Vec<u8> = hasher.finalize().to_vec();
        let hash = base64::encode(hash);
        let response = es_client
            .index(IndexParts::IndexId("greetings", hash.as_str()))
            .body(json!({
                    "id": hash,
                    "value": value,
                }))
            .send()
            .await
            ;
        debug!("Elastic Search response: {:?}", response);
    }
    Ok(())
}
