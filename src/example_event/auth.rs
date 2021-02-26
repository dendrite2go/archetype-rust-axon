use anyhow::{Context,Result,anyhow};
use core::convert::TryFrom;
use dendrite::axon_utils::{AsyncApplicableTo, AxonServerHandle, TheHandlerRegistry, TokenStore, empty_handler_registry, event_processor};
use lazy_static::lazy_static;
use log::{debug,error};
use prost::Message;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use pem;
use rsa::RSAPrivateKey;
use crate::dendrite_config;
use crate::dendrite_config::{CredentialsAddedEvent, CredentialsRemovedEvent, KeyManagerAddedEvent, KeyManagerRemovedEvent, TrustedKeyAddedEvent, TrustedKeyRemovedEvent};

struct AuthSettings {
    key_managers: HashMap<String,String>,
    trusted_keys: HashMap<String,String>,
    credentials: HashMap<String,String>,
    private_key: Option<RSAPrivateKey>,
    private_key_name: String,
}

#[derive(Clone)]
struct AuthQueryModel {
    auth_settings: Arc<Mutex<AuthSettings>>,
}

#[tonic::async_trait]
impl TokenStore for AuthQueryModel {
    async fn store_token(&self, _token: i64) {
    }

    async fn retrieve_token(&self) -> Result<i64> {
        Ok(0)
    }
}

lazy_static! {
    static ref AUTH: AuthQueryModel = AuthQueryModel {
        auth_settings: Arc::new(Mutex::new(AuthSettings {
            key_managers: HashMap::new(),
            trusted_keys: HashMap::new(),
            credentials: HashMap::new(),
            private_key: None,
            private_key_name: "".to_string(),
        }))
    };
}

/// Handles auth events.
///
/// Constructs an event handler registry and delegates to function `event_processor`.
pub async fn process_events(axon_server_handle : AxonServerHandle) {
    if let Err(e) = internal_process_events(axon_server_handle).await {
        error!("Error while handling commands: {:?}", e);
    }
    debug!("Stopped handling auth events");
}

async fn internal_process_events(axon_server_handle : AxonServerHandle) -> Result<()> {
    let mut event_handler_registry: TheHandlerRegistry<AuthQueryModel,Option<AuthQueryModel>> = empty_handler_registry();

    event_handler_registry.register(&handle_trusted_key_added_event)?;
    event_handler_registry.register(&handle_trusted_key_removed_event)?;
    event_handler_registry.register(&handle_key_manager_added_event)?;
    event_handler_registry.register(&handle_key_manager_removed_event)?;
    event_handler_registry.register(&handle_credentials_added_event)?;
    event_handler_registry.register(&handle_credentials_removed_event)?;

    event_processor(axon_server_handle, AUTH.clone(), event_handler_registry).await.context("Error while handling commands")
}

#[dendrite_macros::event_handler]
async fn handle_trusted_key_added_event(command: TrustedKeyAddedEvent, _query_model: AuthQueryModel) -> Result<()> {
    if let Some(public_key) = command.public_key.clone() {
        unsafe_set_public_key(public_key)?
    }
    Ok(())
}

#[dendrite_macros::event_handler]
async fn handle_trusted_key_removed_event(command: TrustedKeyRemovedEvent, _query_model: AuthQueryModel) -> Result<()> {
    remove_public_key(&command.name)
}

#[dendrite_macros::event_handler]
async fn handle_key_manager_added_event(command: KeyManagerAddedEvent, _query_model: AuthQueryModel) -> Result<()> {
    if let Some(public_key) = command.public_key.clone() {
        unsafe_set_key_manager(public_key)?
    }
    Ok(())
}

#[dendrite_macros::event_handler]
async fn handle_key_manager_removed_event(command: KeyManagerRemovedEvent, _query_model: AuthQueryModel) -> Result<()> {
    remove_key_manager(&command.name)
}

#[dendrite_macros::event_handler]
async fn handle_credentials_added_event(command: CredentialsAddedEvent, _query_model: AuthQueryModel) -> Result<()> {
    if let Some(credentials) = command.credentials.clone() {
        unsafe_set_credentials(credentials)?
    }
    Ok(())
}

#[dendrite_macros::event_handler]
async fn handle_credentials_removed_event(command: CredentialsRemovedEvent, _query_model: AuthQueryModel) -> Result<()> {
    remove_credentials(&command.identifier)
}

pub fn set_private_key(key_name: String, pem_string: String) -> Result<()> {
    let pem = pem::parse(pem_string)?;
    let private_key = RSAPrivateKey::try_from(pem)?;
    // TODO: verify that the private key matches the public key with the same name
    let mut auth_settings = AUTH.auth_settings.lock().map_err(|e| anyhow!(e.to_string()))?;
    auth_settings.private_key_name = key_name;
    auth_settings.private_key = Some(private_key);
    Ok(())
}

fn unsafe_set_public_key(public_key: dendrite_config::PublicKey) -> Result<()> {
    let mut auth_settings = AUTH.auth_settings.lock().map_err(|e| anyhow!(e.to_string()))?;
    auth_settings.trusted_keys.insert(public_key.name, public_key.public_key);
    Ok(())
}

fn unsafe_set_key_manager(public_key: dendrite_config::PublicKey) -> Result<()> {
    let mut auth_settings = AUTH.auth_settings.lock().map_err(|e| anyhow!(e.to_string()))?;
    auth_settings.key_managers.insert(public_key.name, public_key.public_key);
    Ok(())
}

fn unsafe_set_credentials(credentials: dendrite_config::Credentials) -> Result<()> {
    let mut auth_settings = AUTH.auth_settings.lock().map_err(|e| anyhow!(e.to_string()))?;
    auth_settings.credentials.insert(credentials.identifier, credentials.secret);
    Ok(())
}

fn remove_public_key (name: &str) -> Result<()> {
    let mut auth_settings = AUTH.auth_settings.lock().map_err(|e| anyhow!(e.to_string()))?;
    auth_settings.trusted_keys.remove(name);
    Ok(())
}

fn remove_key_manager (name: &str) -> Result<()> {
    let mut auth_settings = AUTH.auth_settings.lock().map_err(|e| anyhow!(e.to_string()))?;
    auth_settings.key_managers.remove(name);
    Ok(())
}

fn remove_credentials(identifier: &str) -> Result<()> {
    let mut auth_settings = AUTH.auth_settings.lock().map_err(|e| anyhow!(e.to_string()))?;
    auth_settings.credentials.remove(identifier);
    Ok(())
}
