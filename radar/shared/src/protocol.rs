use serde::{
    Deserialize,
    Serialize,
};
use typescript_type_def::TypeDef;

use crate::RadarState;

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
    ResponseInitializePublish { session_id: String, version: u32 },
    ResponseSubscribeSuccess {},
    ResponseSessionInvalidId {},

    NotifyRadarState { state: RadarState },
    NotifyViewCount { viewers: usize },
    NotifySessionClosed {},
}

#[derive(Serialize, Deserialize, TypeDef)]
#[serde(rename_all = "kebab-case", tag = "type", content = "payload")]
pub enum C2SMessage {
    InitializePublish { version: u32 },
    InitializeSubscribe { version: u32, session_id: String },

    NotifyRadarState { state: RadarState },

    Disconnect { reason: String },
}

pub enum ClientEvent<T> {
    RecvMessage(T),
    RecvError(anyhow::Error),
    SendError(anyhow::Error),
}
