use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    RadarSettings,
    RadarState,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RadarUpdate {
    Settings { settings: RadarSettings },
    State { state: RadarState },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SubscribeResult {
    Success,
    SessionDoesNotExists,
    // SessionRequiresPassword,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum S2CMessage {
    // Generic responses
    ResponseSuccess,
    ResponseError { error: String },

    ResponseInvalidClientState,
    ResponseInitializePublish { session_id: String, version: u32 },
    ResponseSubscribeSuccess,
    ResponseSessionInvalidId,

    NotifyRadarUpdate { update: RadarUpdate },
    NotifyViewCount { viewers: usize },
    NotifySessionClosed,
}

#[derive(Serialize, Deserialize)]
pub enum C2SMessage {
    InitializePublish { version: u32 },
    InitializeSubscribe { version: u32, session_id: String },

    RadarUpdate { update: RadarUpdate },

    Disconnect { message: String },
}

pub enum ClientEvent<T> {
    RecvMessage(T),
    RecvError(anyhow::Error),
    SendError(anyhow::Error),
}
