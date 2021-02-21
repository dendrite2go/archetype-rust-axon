use std::error::Error;
use log::{debug,info};

use tonic::transport::Server;

use dendrite::axon_utils::platform_worker;
use dendrite_example::example_api::init;
use dendrite_example::example_command::handle_commands;
use dendrite_example::example_event::process_events;
use dendrite_example::example_query::process_queries;
use dendrite_example::grpc_example::greeter_service_server::GreeterServiceServer;
use tonic::{Request, Status};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    info!("Dendrite Example API service started");

    let greeter_server = init().await.unwrap();

    tokio::spawn(platform_worker(greeter_server.axon_server_handle.clone(), "Rustic"));

    tokio::spawn(handle_commands(greeter_server.axon_server_handle.clone()));

    tokio::spawn(process_events(greeter_server.axon_server_handle.clone()));

    tokio::spawn(process_queries(greeter_server.axon_server_handle.clone()));

    let addr = "0.0.0.0:8181".parse()?;
    info!("Starting gRPC server");
    Server::builder()
        .add_service(GreeterServiceServer::with_interceptor(greeter_server, interceptor))
        .serve(addr)
        .await?;

    Ok(())
}

fn interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    let token = match req.metadata().get("authorization") {
        Some(token) => token.to_str().unwrap(),
        None => ""
    };
    debug!("Using token: [{:?}]", token);
    Ok(req)
}