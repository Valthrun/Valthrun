use serde::{
    Deserialize,
    Serialize,
};
use typescript_type_def::TypeDef;

use crate::RadarState;

pub const RADAR_PROTOCOL_VERSION: u32 = 2;

#[derive(Serialize, Deserialize, Clone, Debug, TypeDef)]
pub enum SubscribeResult {
    Success,
    SessionDoesNotExists,
    // SessionRequiresPassword,
}

#[derive(Serialize, Deserialize, Clone, Debug, TypeDef)]
#[serde(rename_all = "kebab-case", tag = "type", content = "payload")]
pub enum S2CMessage {
    // Generic responses
    ResponseSuccess {},
    ResponseError { error: String },

    ResponseInvalidClientState {},
    ResponseInitializePublish { session_id: String },
    ResponseSubscribeSuccess {},
    ResponseSessionInvalidId {},

    NotifyRadarState { state: RadarState },
    NotifyViewCount { viewers: usize },
    NotifySessionClosed {},
}

#[derive(Serialize, Deserialize, TypeDef)]
#[serde(rename_all = "kebab-case", tag = "type", content = "payload")]
pub enum C2SMessage {
    InitializePublish {},
    InitializeSubscribe { session_id: String },

    NotifyRadarState { state: RadarState },

    Disconnect { reason: String },
}

pub enum ClientEvent<T> {
    RecvMessage(T),
    RecvError(anyhow::Error),
    SendError(anyhow::Error),
}

#[derive(Serialize, Deserialize, TypeDef)]
pub enum HandshakeProtocolV1 {
    /*
     * Protocol version 1
     * This protocol does not has an explicit protocol handshake.
     * Instead the version is transmitted in the initialize functions.
     */
    InitializePublish { version: u32 },
    InitializeSubscribe { version: u32 },

    /*
     * We only need the error response, as we do not support protocol v1 any more
     */
    ResponseError { error: String },
}

#[derive(Serialize, Deserialize, TypeDef)]
#[serde(
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    tag = "type",
    content = "payload"
)]
pub enum HandshakeProtocolV2 {
    RequestInitialize { client_version: u32 },

    ResponseSuccess { server_version: u32 },
    ResponseIncompatible { supported_versions: Vec<u32> },
    ResponseGenericFailure { message: String },
}

#[derive(Serialize, Deserialize, TypeDef)]
#[serde(untagged)]
pub enum HandshakeMessage {
    V1(HandshakeProtocolV1),
    V2(HandshakeProtocolV2),
}
