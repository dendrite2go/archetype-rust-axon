use anyhow::{Context,Result,anyhow};
use async_lock::Mutex;
use dendrite::axon_utils::{AggregateContext, AggregateContextTrait, AggregateDefinition, ApplicableTo, AxonConnection, AxonServerHandle, HandlerRegistry, SerializedObject, TheHandlerRegistry, command_worker, create_aggregate_definition, empty_handler_registry, empty_aggregate_registry};
use log::{debug,error};
use prost::{Message};
use std::sync::Arc;
use std::ops::Deref;
use crate::grpc_example::{Acknowledgement,Empty,GreetCommand,GreetedEvent,GreeterProjection,RecordCommand,StartedRecordingEvent,StopCommand,StoppedRecordingEvent};

/// Handles commands for the example application.
///
/// Constructs an aggregate registry and delegates to function `command_worker`.
pub async fn handle_commands(axon_server_handle : AxonServerHandle) {
    if let Err(e) = internal_handle_commands(axon_server_handle).await {
        error!("Error while handling commands: {:?}", e);
    }
    debug!("Stopped handling commands for example application");
}

async fn internal_handle_commands(axon_server_handle : AxonServerHandle) -> Result<()> {
    debug!("Handle commands for example application");
    let axon_connection = AxonConnection {
        id: axon_server_handle.display_name,
        conn: axon_server_handle.conn,
    };
    debug!("Axon connection: {:?}", axon_connection);

    let mut sourcing_handler_registry = empty_handler_registry();
    let mut command_handler_registry: TheHandlerRegistry<Arc<Mutex<AggregateContext<GreeterProjection>>>,SerializedObject> = empty_handler_registry();

    sourcing_handler_registry.insert_with_output(
        "GreetedEvent",
        &GreetedEvent::decode,
        &(|c, p| Box::pin(handle_sourcing_event(Box::from(c), p)))
    )?;

    sourcing_handler_registry.insert_with_output(
        "StoppedRecordingEvent",
        &StoppedRecordingEvent::decode,
        &(|c, p| Box::pin(handle_sourcing_event(Box::from(c), p)))
    )?;

    sourcing_handler_registry.insert_with_output(
        "StartedRecordingEvent",
        &StartedRecordingEvent::decode,
        &(|c, p| Box::pin(handle_sourcing_event(Box::from(c), p)))
    )?;

    command_handler_registry.register(&handle_greet_command)?;

    command_handler_registry.register(&handle_record_command)?;

    command_handler_registry.register(&handle_stop_command)?;

    let aggregate_definition: AggregateDefinition<GreeterProjection> = create_aggregate_definition(
        "GreeterProjection".to_string(),
        Box::from(empty_projection as fn() -> GreeterProjection),
        command_handler_registry,
        sourcing_handler_registry
    );

    let mut aggregate_registry = empty_aggregate_registry();
    aggregate_registry.handlers.insert(aggregate_definition.projection_name.clone(), Arc::new(Arc::new(aggregate_definition)));

    command_worker(axon_connection, &mut aggregate_registry).await.context("Error while handling commands")
}

fn empty_projection() -> GreeterProjection {
    let mut projection = GreeterProjection::default();
    projection.is_recording = true;
    projection
}

async fn handle_sourcing_event<T: ApplicableTo<P>,P: Clone>(event: Box<T>, projection: P) -> Result<Option<P>> {
    let mut p = projection.clone();
    event.apply_to(&mut p)?;
    Ok(Some(p))
}

impl ApplicableTo<GreeterProjection> for GreetedEvent {

    fn apply_to(self: &Self, projection: &mut GreeterProjection) -> Result<()> {
        debug!("Apply greeted event to GreeterProjection: {:?}", projection.is_recording);
        Ok(())
    }

    fn box_clone(self: &Self) -> Box<dyn ApplicableTo<GreeterProjection>> {
        Box::from(GreetedEvent::clone(self))
    }
}

impl ApplicableTo<GreeterProjection> for StartedRecordingEvent {

    fn apply_to(self: &Self, projection: &mut GreeterProjection) -> Result<()> {
        debug!("Apply StartedRecordingEvent to GreeterProjection: {:?}", projection.is_recording);
        projection.is_recording = true;
        Ok(())
    }

    fn box_clone(self: &Self) -> Box<dyn ApplicableTo<GreeterProjection>> {
        Box::from(StartedRecordingEvent::clone(self))
    }
}

impl ApplicableTo<GreeterProjection> for StoppedRecordingEvent {

    fn apply_to(self: &Self, projection: &mut GreeterProjection) -> Result<()> {
        debug!("Apply StoppedRecordingEvent to GreeterProjection: {:?}", projection.is_recording);
        projection.is_recording = false;
        Ok(())
    }

    fn box_clone(self: &Self) -> Box<dyn ApplicableTo<GreeterProjection>> {
        Box::from(StoppedRecordingEvent::clone(self))
    }
}

#[dendrite_macros::command_handler]
async fn handle_greet_command(command: GreetCommand, aggregate_context: &mut AggregateContext<GreeterProjection>) -> Result<Option<Acknowledgement>> {
    let greeting = command.message;
    let message = greeting.clone().map(|g| g.message).unwrap_or("-/-".to_string());
    if message == "ERROR" {
        return Err(anyhow!("Panicked at reading 'ERROR'"));
    }

    let projection = aggregate_context.get_projection("xxx").await?;
    if !projection.is_recording {
        debug!("Not recording, so no events emitted nor acknowledgement returned");
        return Ok(None);
    }
    debug!("Recording, so proceed");

    aggregate_context
        .emit("GreetedEvent", Box::from(GreetedEvent {
            message: greeting,
        }))?;

    Ok(Some(Acknowledgement {
        message: format!("ACK! {}", message),
    }))
}

#[dendrite_macros::command_handler]
async fn handle_record_command(command: RecordCommand, aggregate_context: &mut AggregateContext<GreeterProjection>) -> Result<Option<Empty>> {
    let projection = aggregate_context.get_projection("xxx").await?;
    debug!("Record command handler: {:?}", command);
    if projection.is_recording {
        debug!("Unnecessary RecordCommand");
        return Ok(None)
    }
    aggregate_context.emit("StartedRecordingEvent", Box::from(StartedRecordingEvent {}))?;
    Ok(Some(Empty::default()))
}

#[dendrite_macros::command_handler]
async fn handle_stop_command(command: StopCommand, aggregate_context: &mut AggregateContext<GreeterProjection>) -> Result<Option<Empty>> {
    let projection = aggregate_context.get_projection("xxx").await?;
    debug!("Stop command handler: {:?}", command);
    if !projection.is_recording {
        debug!("Unnecessary StopCommand");
        return Ok(None)
    }
    aggregate_context.emit("StoppedRecordingEvent", Box::from(StoppedRecordingEvent {}))?;
    Ok(Some(Empty::default()))
}
