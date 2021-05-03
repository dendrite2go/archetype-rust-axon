use crate::elastic_search_utils::wait_for_elastic_search;
use crate::proto_example::{Greeting, SearchQuery, SearchResponse};
use anyhow::{Context, Result};
use dendrite::axon_server::query::QueryRequest;
use dendrite::axon_utils::{
    axon_serialize, empty_handler_registry, query_processor, AxonServerHandle, HandlerRegistry,
    QueryContext, QueryResult, TheHandlerRegistry,
};
use elasticsearch::{Elasticsearch, SearchParts};
use log::{debug, error};
use prost::Message;

#[derive(Clone)]
struct ExampleQueryContext {
    es_client: Elasticsearch,
}

impl QueryContext for ExampleQueryContext {}

/// Handles queries for the example application.
///
/// Constructs an query handler registry and delegates to function `query_processor`.
pub async fn process_queries(axon_server_handle: AxonServerHandle) {
    if let Err(e) = internal_process_queries(axon_server_handle).await {
        error!("Error while handling queries: {:?}", e);
    }
    debug!("Stopped handling commands for example application");
}

async fn internal_process_queries(axon_server_handle: AxonServerHandle) -> Result<()> {
    let client = wait_for_elastic_search().await?;
    debug!("Elastic Search client: {:?}", client);

    let query_context = ExampleQueryContext { es_client: client };

    let mut query_handler_registry: TheHandlerRegistry<
        ExampleQueryContext,
        QueryRequest,
        QueryResult,
    > = empty_handler_registry();

    query_handler_registry.register(&handle_search_query)?;

    query_processor(axon_server_handle, query_context, query_handler_registry)
        .await
        .context("Error while handling queries")
}

#[dendrite_macros::query_handler]
async fn handle_search_query(
    search_query: SearchQuery,
    query_model: ExampleQueryContext,
) -> Result<Option<QueryResult>> {
    let search_response = query_model
        .es_client
        .search(SearchParts::Index(&["greetings"]))
        .q(&search_query.query)
        ._source(&["value"])
        .send()
        .await?;
    let json_value: serde_json::Value = search_response.json().await?;
    debug!("Search response: {:?}", json_value);
    let hits = &json_value["hits"]["hits"];
    debug!("Hits: {:?}", hits);
    let mut greetings = Vec::new();
    if let serde_json::Value::Array(hits) = hits {
        for document in hits {
            if let serde_json::Value::String(message) = &document["_source"]["value"] {
                let greeting = Greeting {
                    message: message.clone(),
                };
                greetings.push(greeting);
            }
        }
    }
    let greeting = Greeting {
        message: "Test!".to_string(),
    };
    greetings.push(greeting);
    let response = SearchResponse { greetings };
    let result = axon_serialize("SearchResponse", &response)?;
    let query_result = QueryResult {
        payload: Some(result),
    };
    Ok(Some(query_result))
}
