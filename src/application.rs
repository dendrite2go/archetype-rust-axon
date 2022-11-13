use dendrite::auth as dendrite_auth;
use dendrite::axon_utils::{AxonServerHandle, AxonServerHandleAsyncTrait, platform_worker_for};
use dendrite::elasticsearch::replica;
use log::{debug, error, info, warn};
use prost::Message;
use std::error::Error;
use anyhow::anyhow;
use futures_util::FutureExt;
use tonic::transport::Server;
use tonic::{Request, Status};
use uuid::Uuid;
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
    let axon_server_handle = &greeter_server.axon_server_handle.clone();

    axon_server_handle.spawn("Platform", platform_worker_for("Rustic"))?;

    axon_server_handle.spawn_ref("Command", &handle_commands)?;
    axon_server_handle.spawn_ref("Event", &process_events)?;

    let transcoders = replica::Transcoders::new()
        .insert_ref("GreetedEvent", &GreetedEvent::decode)
        .insert_ref("StartedRecordingEvent", &StartedRecordingEvent::decode)
        .insert_ref("StoppedRecordingEvent", &StoppedRecordingEvent::decode)
        .insert_ref("PropertyChangedEvent", &PropertyChangedEvent::decode);
    axon_server_handle.spawn("Replica",replica::process_events_with(transcoders))?;

    trusted_generated::init()?;
    axon_server_handle.spawn_ref("Auth",&dendrite_auth::process_events)?;

    axon_server_handle.spawn_ref("Query",&process_queries)?;

    let addr = "0.0.0.0:8181".parse()?;
    info!("Starting gRPC server");
    let server = Server::builder()
        .add_service(GreeterServiceServer::with_interceptor(
            greeter_server,
            interceptor,
        ))
        .serve(addr);
    axon_server_handle.spawn("gRPC server", Box::new(|handle,_control_channel| {
        Box::pin(server.then(|result| send_termination_notification(handle, result.map_err(|e| anyhow!(e.to_string())), "gRPC Server", Uuid::new_v4())))
    }))?;

    axon_server_handle.join_workers().await?;
    Ok(())
}

async fn send_termination_notification<S: Into<String>>(handle: AxonServerHandle, result: anyhow::Result<()>, label: S, id: Uuid) {
    let label = &*label.into();
    if let Err(e) = result {
        error!("Error in worker: {:?}: {:?}", label, e)
    }
    if let Err(e) = handle.notify.send(id).await {
        error!("Termination notification send failed: {:?}: {:?}", label, e)
    }
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
