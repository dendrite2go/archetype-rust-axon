use dendrite::auth as dendrite_auth;
use dendrite::axon_utils::platform_worker;
use dendrite::elasticsearch::replica::Transcoders;
use log::{debug, info, warn};
use prost::Message;
use std::error::Error;
use tonic::transport::Server;
use tonic::{Request, Status};

use crate::example_api::init;
use crate::example_command::handle_commands;
use crate::example_event::{process_events, trusted_generated};
use crate::example_query::process_queries;
use crate::proto_example::greeter_service_server::GreeterServiceServer;
use crate::proto_example::{
    GreetedEvent, PropertyChangedEvent, StartedRecordingEvent, StoppedRecordingEvent,
};

pub async fn application() -> Result<(), Box<dyn Error>> {
    let greeter_server = init().await.unwrap();

    tokio::spawn(platform_worker(
        greeter_server.axon_server_handle.clone(),
        "Rustic",
    ));

    tokio::spawn(handle_commands(greeter_server.axon_server_handle.clone()));

    tokio::spawn(process_events(greeter_server.axon_server_handle.clone()));

    let transcoders = Transcoders::new()
        .insert("GreetedEvent", Box::new(GreetedEvent::decode))
        .insert(
            "StartedRecordingEvent",
            Box::new(StartedRecordingEvent::decode),
        )
        .insert(
            "StoppedRecordingEvent",
            Box::new(StoppedRecordingEvent::decode),
        )
        .insert(
            "PropertyChangedEvent",
            Box::new(PropertyChangedEvent::decode),
        );
    tokio::spawn(dendrite::elasticsearch::replica::process_events(
        greeter_server.axon_server_handle.clone(),
        transcoders,
    ));

    trusted_generated::init()?;
    tokio::spawn(dendrite_auth::process_events(
        greeter_server.axon_server_handle.clone(),
    ));

    tokio::spawn(process_queries(greeter_server.axon_server_handle.clone()));

    let addr = "0.0.0.0:8181".parse()?;
    info!("Starting gRPC server");
    Server::builder()
        .add_service(GreeterServiceServer::with_interceptor(
            greeter_server,
            interceptor,
        ))
        .serve(addr)
        .await?;

    Ok(())
}

fn interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    let token = match req.metadata().get("authorization") {
        Some(token) => token.to_str().unwrap(),
        None => "",
    };
    debug!("Using token: [{:?}]", token);
    let credentials = dendrite_auth::verify_jwt(token);
    match credentials {
        Ok(claims) => debug!("Credentials: [{:?}]", claims),
        Err(error) => warn!("JWT parsing error: {:?}", error),
    };
    Ok(req)
}
