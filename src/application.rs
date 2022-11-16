use dendrite::auth as dendrite_auth;
use dendrite::axon_utils::{AxonServerHandle, AxonServerHandleAsyncTrait, platform_worker_for, WorkerControl};
use dendrite::elasticsearch::replica;
use log::{debug, error, info, warn};
use prost::Message;
use std::error::Error;
use anyhow::anyhow;
use async_channel::{bounded, Receiver};
use futures_util::FutureExt;
use tokio::signal::unix::{signal,SignalKind};
use tonic::transport::Server;
use tonic::{Request, Status};
use uuid::Uuid;
use crate::example_api::{GreeterServer, init};
use crate::example_command::handle_commands;
use crate::example_event::{process_events, trusted_generated};
use crate::example_query::process_queries;
use crate::proto_example::greeter_service_server::GreeterServiceServer;
use crate::proto_example::{
    GreetedEvent, PropertyChangedEvent, StartedRecordingEvent, StoppedRecordingEvent,
};

pub async fn application() -> Result<(), Box<dyn Error>> {
    let signal_stream = signal(SignalKind::terminate())?;
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

    info!("Starting gRPC server");
    let (tx, rx) = bounded(10);
    let id = axon_server_handle.spawn("gRPC server", Box::new(|handle, control_channel| {
        Box::pin(
            run_server(greeter_server, control_channel)
                .then(|result| {
                    send_termination_notification(
                        handle,
                        result.map_err(|e| anyhow!(e.to_string())),
                        "gRPC Server",
                        rx
                    )
                })
        )
    }))?;
    tx.send(id).await.map_err(|e| {
        error!("Error sending server id: {:?}", e);
        e
    })?;

    let mut signal = Some(signal_stream);
    axon_server_handle.join_workers_with_signal(&mut signal).await?;
    Ok(())
}

async fn run_server(greeter_server: GreeterServer, worker_control: WorkerControl) -> anyhow::Result<()> {
    debug!("Run server: {:?}", worker_control.get_label());
    let control_channel = worker_control.get_control_channel().clone();
    let addr = "0.0.0.0:8181".parse()?;
    Server::builder()
        .add_service(GreeterServiceServer::with_interceptor(
            greeter_server,
            interceptor,
        ))
        .serve_with_shutdown(addr, control_channel.recv().map(|_r|()))
        .await.map_err(|e| anyhow!(e))
}

async fn send_termination_notification<S: Into<String>>(handle: AxonServerHandle, result: anyhow::Result<()>, label: S, id: Receiver<Uuid>) {
    let label = &*label.into();
    if let Err(e) = result {
        error!("Error in worker: {:?}: {:?}", label, e)
    }
    let id = match id.recv().await {
        Ok(id) =>
            id,
        Err(e) => {
            error!("No id received: {:?}", e);
            Uuid::new_v4()
        }
    };
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
