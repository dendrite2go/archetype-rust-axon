use crate::proto_example::{GreetedEvent, Greeting};
use anyhow::{Context, Result};
use dendrite::axon_server::event::Event;
use dendrite::axon_utils::{
    empty_handler_registry, event_processor, AsyncApplicableTo, AxonServerHandle,
    TheHandlerRegistry, TokenStore,
};
use dendrite::elasticsearch::{
    create_elastic_query_model, wait_for_elastic_search, ElasticQueryModel,
};
use dendrite::macros as dendrite_macros;
use dendrite::register;
use elasticsearch::{Elasticsearch, IndexParts};
use log::{debug, error};
use prost::Message;
use serde_json::json;
use sha2::{Digest, Sha256};

pub mod trusted_generated;

#[derive(Clone)]
struct ExampleQueryModel(ElasticQueryModel);

#[tonic::async_trait]
impl TokenStore for ExampleQueryModel {
    async fn store_token(&self, token: i64) {
        self.0.store_token(token).await;
    }

    async fn retrieve_token(&self) -> Result<i64> {
        self.0.retrieve_token().await
    }
}

impl ExampleQueryModel {
    pub fn get_client(&self) -> &Elasticsearch {
        &self.0.get_client()
    }
}

/// Handles events for the example application.
///
/// Constructs an event handler registry and delegates to function `event_processor`.
pub async fn process_events(axon_server_handle: AxonServerHandle) {
    if let Err(e) = internal_process_events(axon_server_handle).await {
        error!("Error while handling commands: {:?}", e);
    }
    debug!("Stopped handling commands for example application");
}

async fn internal_process_events(axon_server_handle: AxonServerHandle) -> Result<()> {
    let client = wait_for_elastic_search().await?;
    debug!("Elastic Search client: {:?}", client);

    let elastic_query_model = create_elastic_query_model(client, "greeting".to_string());
    let query_model = ExampleQueryModel(elastic_query_model);

    let mut event_handler_registry: TheHandlerRegistry<
        ExampleQueryModel,
        Event,
        Option<ExampleQueryModel>,
    > = empty_handler_registry();

    register!(event_handler_registry, handle_greeted_event)?;

    event_processor(axon_server_handle, query_model, event_handler_registry)
        .await
        .context("Error while handling commands")
}

#[dendrite_macros::event_handler]
pub async fn handle_greeted_event(
    event: GreetedEvent,
    query_model: ExampleQueryModel,
    message: Event,
) -> Result<()> {
    debug!(
        "Apply greeted event to ExampleQueryModel: {:?}",
        message.timestamp
    );
    let es_client = query_model.get_client();
    if let Some(Greeting { message }) = &event.message {
        let mut hasher = Sha256::new();
        Digest::update(&mut hasher, message);
        let hash: Vec<u8> = hasher.finalize().to_vec();
        let hash = base64::encode(hash);
        let response = es_client
            .index(IndexParts::IndexId("greetings", hash.as_str()))
            .body(json!({
                "id": hash,
                "value": message.to_string(),
            }))
            .send()
            .await;
        debug!("Elastic Search response: {:?}", response);
    }
    Ok(())
}
