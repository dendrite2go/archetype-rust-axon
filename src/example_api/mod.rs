use crate::proto_example::greeter_service_server::GreeterService;
use crate::proto_example::{
    Acknowledgement, Empty, GreetCommand, GreetedEvent, Greeting, RecordCommand, SearchQuery,
    SearchResponse, StopCommand,
};
use anyhow::{Error, Result};
use bytes::Bytes;
use dendrite::axon_utils::{
    init_command_sender, query_events, AxonServerHandle, CommandSink, QuerySink,
};
use dendrite::intellij_work_around::Debuggable;
use futures_core::stream::Stream;
use log::debug;
use prost::Message;
use std::fmt::Debug;
use std::pin::Pin;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};

/// Carries an `AxonServerHandle` and implements the `prost` generated `GreeterService`.
///
/// The implementation uses implementations of `CommandSink` and `QuerySink` to send commands and queries to AxonServer.
#[derive(Debug)]
pub struct GreeterServer {
    pub axon_server_handle: AxonServerHandle,
}

#[tonic::async_trait]
impl GreeterService for GreeterServer {
    async fn greet(&self, request: Request<Greeting>) -> Result<Response<Acknowledgement>, Status> {
        let inner_request = request.into_inner();
        debug!(
            "Got a greet request: {:?}",
            Debuggable::from(&inner_request)
        );
        let result_message = inner_request.message.clone();

        let command = GreetCommand {
            aggregate_identifier: "xxx".to_string(),
            message: Some(inner_request),
        };

        if let Some(serialized) = self
            .axon_server_handle
            .send_command("GreetCommand", &command)
            .await
            .map_err(to_status)?
        {
            let reply_from_command_handler =
                Message::decode(Bytes::from(serialized.data)).map_err(decode_error_to_status)?;
            debug!(
                "Reply from command handler: {:?}",
                Debuggable::from(&reply_from_command_handler)
            );
            return Ok(Response::new(reply_from_command_handler));
        }

        let default_reply = Acknowledgement {
            message: format!("Hello {}!", result_message).into(),
        };

        Ok(Response::new(default_reply))
    }

    async fn record(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        debug!(
            "Got a record request: {:?}",
            Debuggable::from(&request.into_inner())
        );

        let command = RecordCommand {
            aggregate_identifier: "xxx".to_string(),
        };

        self.axon_server_handle
            .send_command("RecordCommand", &command)
            .await
            .map_err(to_status)?;

        let reply = Empty {};

        Ok(Response::new(reply))
    }

    async fn stop(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        debug!(
            "Got a stop request: {:?}",
            Debuggable::from(&request.into_inner())
        );

        let command = StopCommand {
            aggregate_identifier: "xxx".to_string(),
        };

        self.axon_server_handle
            .send_command("StopCommand", &command)
            .await
            .map_err(to_status)?;

        let reply = Empty {};

        Ok(Response::new(reply))
    }

    type GreetingsStream =
        Pin<Box<dyn Stream<Item = Result<Greeting, Status>> + Send + Sync + 'static>>;

    async fn greetings(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::GreetingsStream>, Status> {
        let events = query_events(&self.axon_server_handle, "xxx")
            .await
            .map_err(to_status)?;
        let (tx, mut rx): (
            mpsc::Sender<Result<Greeting>>,
            mpsc::Receiver<Result<Greeting>>,
        ) = mpsc::channel(4);

        tokio::spawn(async move {
            for event in &events[..] {
                let event = event.clone();
                if let Some(payload) = event.payload {
                    if payload.r#type == "GreetedEvent" {
                        let greeted_event_message = GreetedEvent::decode(Bytes::from(payload.data))
                            .ok()
                            .map(|e| e.message);
                        if let Some(greeting) = greeted_event_message.flatten() {
                            debug!("Greeting: {:?}", greeting);
                            tx.send(Ok(greeting)).await.ok();
                        }
                    }
                }
            }
            let greeting = Greeting {
                message: "End of stream -oo-".to_string(),
            };
            debug!("End of stream: {:?}", Debuggable::from(&greeting));
            tx.send(Ok(greeting)).await.ok();
        });

        let output = async_stream::try_stream! {
            while let Some(Ok(value)) = rx.recv().await {
                yield value as Greeting;
            }
        };

        Ok(Response::new(Box::pin(output) as Self::GreetingsStream))
    }

    type SearchStream =
        Pin<Box<dyn Stream<Item = Result<Greeting, Status>> + Send + Sync + 'static>>;

    async fn search(
        &self,
        request: Request<SearchQuery>,
    ) -> Result<Response<Self::SearchStream>, Status> {
        let (tx, mut rx): (
            mpsc::Sender<Result<Greeting>>,
            mpsc::Receiver<Result<Greeting>>,
        ) = mpsc::channel(4);
        let query = request.into_inner();
        let query_response = self
            .axon_server_handle
            .send_query("SearchQuery", &query)
            .await
            .map_err(to_status)?;

        tokio::spawn(async move {
            for serialized_object in query_response {
                if let Ok(search_response) =
                    SearchResponse::decode(Bytes::from(serialized_object.data))
                {
                    debug!("Search response: {:?}", search_response);
                    for greeting in search_response.greetings {
                        debug!("Greeting: {:?}", greeting);
                        tx.send(Ok(greeting)).await.ok();
                    }
                }
                debug!("Next!");
            }
            debug!("Done!")
        });

        let output = async_stream::try_stream! {
            while let Some(Ok(value)) = rx.recv().await {
                yield value as Greeting;
            }
        };

        Ok(Response::new(Box::pin(output) as Self::SearchStream))
    }
}

/// Initialises a `GreeterServer`.
pub async fn init() -> Result<GreeterServer> {
    init_command_sender()
        .await
        .map(|command_sink| GreeterServer {
            axon_server_handle: command_sink,
        })
}

fn to_status(e: Error) -> Status {
    Status::unknown(e.to_string())
}

fn decode_error_to_status(e: prost::DecodeError) -> Status {
    Status::unknown(e.to_string())
}
