use crate::proto_example::GreetedEvent;
use anyhow::{Context,Result};
use dendrite::mongodb::{MongoCollectionQueryModel, create_mongodb_collection_query_model, wait_for_mongodb};
use dendrite::axon_server::event::Event;
use dendrite::axon_utils::{AsyncApplicableTo,AxonServerHandle,TheHandlerRegistry,TokenStore,WorkerControl,empty_handler_registry,event_processor};
use dendrite::macros as dendrite_macros;
use dendrite::register;
use log::{debug,error};
use mongodb::bson::doc;
use mongodb::{bson, Collection};
use mongodb::options::ReplaceOptions;
use prost::Message;

#[derive(Clone)]
struct ExampleQueryModel(MongoCollectionQueryModel);

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
    pub fn get_collection(&self) -> &Collection<bson::Document> {
        &self.0.get_collection()
    }
}

/// Handles events.
///
/// Constructs an event handler registry and delegates to function `event_processor`.
pub async fn process_events_mongo(url: &str, axon_server_handle: AxonServerHandle, worker_control: WorkerControl) {
    if let Err(e) = internal_process_events(url, axon_server_handle, worker_control).await {
        error!("Error while handling events for Mongo Query Model: {:?}", e);
    }
    debug!("Stopped handling events for Mongo Query Model");
}

async fn internal_process_events(url: &str, axon_server_handle: AxonServerHandle, worker_control: WorkerControl) -> Result<()> {
    let client = wait_for_mongodb(url, "Example").await?;
    debug!("Elastic Search client: {:?}", client);

    let mongo_query_model = create_mongodb_collection_query_model("example", client, "grok", "greeting");
    let query_model = ExampleQueryModel(mongo_query_model);

    let mut event_handler_registry: TheHandlerRegistry<
        ExampleQueryModel,
        Event,
        Option<ExampleQueryModel>,
    > = empty_handler_registry();

    register!(event_handler_registry, handle_greeted_event)?;

    event_processor(axon_server_handle, query_model, event_handler_registry, worker_control)
        .await
        .context("Error while handling events for Mongo Query Model")
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
    let object_id = message.message_identifier;
    let greeting = event.message.map(|g|g.message).unwrap_or_default();
    let filter = doc!(
        "_id": object_id.clone(),
    );
    let object = doc!(
        "_id": object_id,
        "greeting": greeting,
    );
    let mut replace_options = ReplaceOptions::default();
    replace_options.upsert = Some(true);
    query_model.get_collection().replace_one(filter, object, replace_options).await?;
    Ok(())
}