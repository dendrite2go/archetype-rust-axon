use anyhow::{Context,Result,anyhow};
use dendrite::axon_utils::{ApplicableTo, AxonConnection, AxonServerHandle, EmitApplicableEventsAndResponse, HandlerRegistry, command_worker, create_aggregate_definition, emit, emit_events_and_response, empty_handler_registry, empty_aggregate_registry};
use log::{debug,error};
use prost::{Message};
use crate::grpc_example::{Acknowledgement,GreetCommand,GreetedEvent,GreeterProjection,RecordCommand,StartedRecordingEvent,StopCommand,StoppedRecordingEvent};

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

    let mut aggregate_id_extractor_registry = empty_handler_registry();
    let mut sourcing_handler_registry = empty_handler_registry();
    let mut command_handler_registry = empty_handler_registry();

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

    aggregate_id_extractor_registry.insert_with_output(
        "GreetCommand",
        &GreetCommand::decode,
        &(|_, _| Box::pin(fixed_aggregate_id()))
    )?;

    command_handler_registry.insert_with_output(
        "GreetCommand",
        &GreetCommand::decode,
        &(|c, p| Box::pin(handle_greet_command(c, p)))
    )?;

    aggregate_id_extractor_registry.insert_with_output(
        "RecordCommand",
        &RecordCommand::decode,
        &(|_, _| Box::pin(fixed_aggregate_id()))
    )?;

    command_handler_registry.insert_with_output(
        "RecordCommand",
        &RecordCommand::decode,
        &(|c, p| Box::pin(handle_record_command(c, p)))
    )?;

    aggregate_id_extractor_registry.insert_with_output(
        "StopCommand",
        &StopCommand::decode,
        &(|_, _| Box::pin(fixed_aggregate_id()))
    )?;

    command_handler_registry.insert_with_output(
        "StopCommand",
        &StopCommand::decode,
        &(|c, p| Box::pin(handle_stop_command(c, p)))
    )?;

    let empty_projection = Box::new(|| {
        let mut projection = GreeterProjection::default();
        projection.is_recording = true;
        projection
    });

    let aggregate_definition = create_aggregate_definition(
        "GreeterProjection".to_string(),
        empty_projection,
        aggregate_id_extractor_registry,
        command_handler_registry,
        sourcing_handler_registry
    );

    let mut aggregate_registry = empty_aggregate_registry();
    aggregate_registry.handlers.insert(aggregate_definition.projection_name.clone(), Box::from(aggregate_definition));

    command_worker(axon_connection, &mut aggregate_registry).await.context("Error while handling commands")
}

async fn handle_sourcing_event<T: ApplicableTo<P>,P: Clone>(event: Box<T>, projection: P) -> Result<Option<P>> {
    let mut p = projection.clone();
    event.apply_to(&mut p)?;
    Ok(Some(p))
}

async fn fixed_aggregate_id() -> Result<Option<String>> {
    Ok(Some("xxx".to_string()))
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

async fn handle_greet_command (command: GreetCommand, projection: GreeterProjection) -> Result<Option<EmitApplicableEventsAndResponse<GreeterProjection>>> {
    debug!("Greet command handler: {:?}", command);
    if !projection.is_recording {
        debug!("Not recording, so no events emitted nor acknowledgement returned");
        return Ok(None);
    }
    debug!("Recording, so proceed");
    let greeting = command.message;
    let message = greeting.clone().map(|g| g.message).unwrap_or("-/-".to_string());
    if message == "ERROR" {
        return Err(anyhow!("Panicked at reading 'ERROR'"));
    }
    let mut emit_events = emit_events_and_response("Acknowledgement", &Acknowledgement {
        message: format!("ACK! {}", message),
    })?;
    emit(&mut emit_events, "GreetedEvent", Box::from(GreetedEvent {
        message: greeting,
    }))?;
    debug!("Emit events and response: {:?}", emit_events);
    Ok(Some(emit_events))
}

async fn handle_record_command (command: RecordCommand, projection: GreeterProjection) -> Result<Option<EmitApplicableEventsAndResponse<GreeterProjection>>> {
    debug!("Record command handler: {:?}", command);
    if projection.is_recording {
        return Ok(None)
    }
    let mut emit_events = emit_events_and_response("Empty", &())?;
    emit(&mut emit_events, "StartedRecordingEvent", Box::from(StartedRecordingEvent {}))?;
    Ok(Some(emit_events))
}

async fn handle_stop_command (command: StopCommand, projection: GreeterProjection) -> Result<Option<EmitApplicableEventsAndResponse<GreeterProjection>>> {
    debug!("Stop command handler: {:?}", command);
    if !projection.is_recording {
        return Ok(None)
    }
    let mut emit_events = emit_events_and_response("Empty", &())?;
    emit(&mut emit_events, "StoppedRecordingEvent", Box::from(StoppedRecordingEvent {}))?;
    Ok(Some(emit_events))
}
