use crate::proto_example::{
    Acknowledgement, Empty, GreetCommand, GreetedEvent, GreeterProjection, RecordCommand,
    StartedRecordingEvent, StopCommand, StoppedRecordingEvent,
};
use anyhow::{anyhow, Context, Result};
use async_lock::Mutex;
use dendrite::axon_server::command::Command;
use dendrite::axon_utils::{command_worker, create_aggregate_definition, empty_aggregate_registry, empty_handler_registry, AggregateContext, AggregateContextTrait, AggregateDefinition, AggregateRegistry, ApplicableTo, AxonServerHandle, HandlerRegistry, SerializedObject, TheHandlerRegistry, WorkerControl};
use dendrite::intellij_work_around::Debuggable;
use dendrite::macros as dendrite_macros;
use log::{debug, error};
use prost::Message;
use std::ops::Deref;
use std::sync::Arc;

/// Handles commands.
///
/// Constructs an aggregate registry and delegates to function `command_worker`.
pub async fn handle_commands(axon_server_handle: AxonServerHandle, worker_control: WorkerControl) {
    if let Err(e) = internal_handle_commands(axon_server_handle, worker_control).await {
        error!("Error while handling commands: {:?}", e);
    }
    debug!("Stopped handling commands");
}

async fn internal_handle_commands(axon_server_handle: AxonServerHandle, worker_control: WorkerControl) -> Result<()> {
    debug!("Handle commands: {:?}", worker_control.get_label());
    debug!("Axon server handle: {:?}", &axon_server_handle);

    let mut sourcing_handler_registry = empty_handler_registry();
    let mut command_handler_registry: TheHandlerRegistry<
        Arc<Mutex<AggregateContext<GreeterProjection>>>,
        Command,
        SerializedObject,
    > = empty_handler_registry();

    command_handler_registry.register(&handle_greet_command)?;
    command_handler_registry.register(&handle_record_command)?;
    command_handler_registry.register(&handle_stop_command)?;

    sourcing_handler_registry.register(&handle_greeted_source_event)?;
    sourcing_handler_registry.register(&handle_started_recording_source_event)?;
    sourcing_handler_registry.register(&handle_stopped_recording_source_event)?;

    let aggregate_definition: AggregateDefinition<GreeterProjection> = create_aggregate_definition(
        "GreeterProjection".to_string(),
        Box::from(empty_projection as fn() -> GreeterProjection),
        command_handler_registry,
        sourcing_handler_registry,
    );

    let mut aggregate_registry = empty_aggregate_registry();
    aggregate_registry.insert(Arc::new(Arc::new(aggregate_definition)))?;

    command_worker(axon_server_handle, &mut aggregate_registry, worker_control)
        .await
        .context("Error while handling commands")
}

fn empty_projection() -> GreeterProjection {
    let mut projection = GreeterProjection::default();
    projection.is_recording = true;
    projection
}

#[dendrite_macros::command_handler]
async fn handle_greet_command(
    command: GreetCommand,
    aggregate_context: &mut AggregateContext<GreeterProjection>,
) -> Result<Option<Acknowledgement>> {
    let message = command
        .message
        .as_ref()
        .map(|g| &*g.message)
        .unwrap_or("-/-");
    if message == "ERROR" {
        return Err(anyhow!("Panicked at reading 'ERROR'"));
    }

    let projection = aggregate_context.get_projection("xxx").await?;
    if !projection.is_recording {
        debug!("Not recording, so no events emitted nor acknowledgement returned");
        return Ok(None);
    }
    debug!("Recording, so proceed");

    let greeting = command.message.clone();
    aggregate_context.emit("GreetedEvent", Box::new(GreetedEvent { message: greeting }))?;

    Ok(Some(Acknowledgement {
        message: format!("ACK! {}", message),
    }))
}

#[dendrite_macros::command_handler]
async fn handle_record_command(
    command: RecordCommand,
    aggregate_context: &mut AggregateContext<GreeterProjection>,
) -> Result<Option<Empty>> {
    let projection = aggregate_context.get_projection("xxx").await?;
    debug!("Record command handler: {:?}", Debuggable::from(&command));
    if projection.is_recording {
        debug!("Unnecessary RecordCommand");
        return Ok(None);
    }
    aggregate_context.emit("StartedRecordingEvent", Box::new(StartedRecordingEvent {}))?;
    Ok(Some(Empty::default()))
}

#[dendrite_macros::command_handler]
async fn handle_stop_command(
    command: StopCommand,
    aggregate_context: &mut AggregateContext<GreeterProjection>,
) -> Result<Option<Empty>> {
    let projection = aggregate_context.get_projection("xxx").await?;
    debug!("Stop command handler: {:?}", Debuggable::from(&command));
    if !projection.is_recording {
        debug!("Unnecessary StopCommand");
        return Ok(None);
    }
    aggregate_context.emit("StoppedRecordingEvent", Box::new(StoppedRecordingEvent {}))?;
    Ok(Some(Empty::default()))
}

#[dendrite_macros::event_sourcing_handler]
fn handle_greeted_source_event(_event: GreetedEvent, projection: GreeterProjection) {
    debug!(
        "Apply greeted event to GreeterProjection: {:?}",
        projection.is_recording
    );
}

#[dendrite_macros::event_sourcing_handler]
fn handle_started_recording_source_event(
    _event: StartedRecordingEvent,
    mut projection: GreeterProjection,
) {
    debug!(
        "Apply StartedRecordingEvent to GreeterProjection: {:?}",
        projection.is_recording
    );
    projection.is_recording = true;
}

#[dendrite_macros::event_sourcing_handler]
fn handle_stopped_recording_source_event(
    _event: StoppedRecordingEvent,
    mut projection: GreeterProjection,
) {
    debug!(
        "Apply StoppedRecordingEvent to GreeterProjection: {:?}",
        projection.is_recording
    );
    projection.is_recording = false;
}
