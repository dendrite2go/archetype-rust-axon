use anyhow::{Context,Result,anyhow};
use core::convert::TryFrom;
use dendrite::axon_utils::{AsyncApplicableTo, AxonServerHandle, TheHandlerRegistry, TokenStore, empty_handler_registry, event_processor};
use jwt::{Header, Token, VerifyWithKey, AlgorithmType, Error};
use jwt::algorithm::AlgorithmType::Rs256;
use jwt::token::Unverified;
use lazy_static::lazy_static;
use log::{debug,error,warn};
use pem;
use prost::Message;
use rsa::{RSAPrivateKey, PublicKey, PaddingScheme};
use rsa::hash::Hash::SHA2_256;
use serde_json::Value;
use sha2::{Digest,Sha256};
use sha2::digest::FixedOutput;
use sshkeys;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::dendrite_config;
use crate::dendrite_config::{CredentialsAddedEvent, CredentialsRemovedEvent, KeyManagerAddedEvent, KeyManagerRemovedEvent, TrustedKeyAddedEvent, TrustedKeyRemovedEvent};

const SEPARATOR: &str = ".";

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
        unchecked_set_public_key(public_key)?
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
        unchecked_set_key_manager(public_key)?
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
        unchecked_set_credentials(credentials)?
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

pub fn unchecked_set_public_key(public_key: dendrite_config::PublicKey) -> Result<()> {
    let mut auth_settings = AUTH.auth_settings.lock().map_err(|e| anyhow!(e.to_string()))?;
    auth_settings.trusted_keys.insert(public_key.name, public_key.public_key);
    Ok(())
}

pub fn unchecked_set_key_manager(public_key: dendrite_config::PublicKey) -> Result<()> {
    let mut auth_settings = AUTH.auth_settings.lock().map_err(|e| anyhow!(e.to_string()))?;
    auth_settings.key_managers.insert(public_key.name, public_key.public_key);
    Ok(())
}

fn unchecked_set_credentials(credentials: dendrite_config::Credentials) -> Result<()> {
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

struct AuthPublicKey(rsa::RSAPublicKey);

impl jwt::VerifyingAlgorithm for AuthPublicKey {
    fn algorithm_type(&self) -> AlgorithmType {
        Rs256
    }

    fn verify_bytes(&self, header: &str, claims: &str, signature: &[u8]) -> Result<bool, Error> {
        let mut hasher = Sha256::new();
        hasher.update(header.as_bytes());
        sha2::Digest::update(&mut hasher, SEPARATOR.as_bytes());
        sha2::Digest::update(&mut hasher, claims.as_bytes());
        let hashed: &[u8] = &hasher.finalize_fixed();
        debug!("Verify signature: {:?}: {:?}", signature, hashed);
        self.0.verify(PaddingScheme::PKCS1v15Sign {hash:Some(SHA2_256)}, hashed, signature)
            .map_err(|e| {
                warn!("Error during verification of JWT: {:?}", e);
                Error::InvalidSignature
            })?;
        Ok(true)
    }
}

pub fn verify_jwt(jwt: &str) -> Result<HashMap<String,Value>> {
    let unverified: Token<Header,HashMap<String,Value>,Unverified> = Token::parse_unverified(jwt)?;
    let header = unverified.header();
    debug!("Header: {:?}", header);
    if let Some(ref key_id) = header.key_id {
        let auth_settings = AUTH.auth_settings.lock().map_err(|e| anyhow!(e.to_string()))?;
        if let Some(key) = auth_settings.trusted_keys.get(key_id) {
            debug!("Key: {:?}", key);
            let key = format!("ssh-rsa {}", key);
            let public_key = sshkeys::PublicKey::from_string(&key)?;
            debug!("SSH public key: {:?}", public_key);
            if let sshkeys::PublicKey{kind: sshkeys::PublicKeyKind::Rsa(sshkeys::RsaPublicKey {n, e}), ..} = public_key {
                let n = rsa::BigUint::from_bytes_be(&n);
                let e = rsa::BigUint::from_bytes_be(&e);
                let public_key = rsa::RSAPublicKey::new(n, e)?;
                debug!("RSH public key: {:?}", public_key);
                let verified = unverified.verify_with_key(&AuthPublicKey(public_key))?;
                return Ok(verified.claims().clone());
            }
        }
    }
    Err(anyhow!("Invalid signature"))
}
