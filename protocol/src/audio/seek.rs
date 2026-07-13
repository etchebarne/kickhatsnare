use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::TransportSnapshot;
use crate::IpcMethod;

pub struct SeekAudio;

impl IpcMethod for SeekAudio {
    const NAME: &'static str = "audio.seek";
    type Params = SeekAudioParams;
    type Result = TransportSnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct SeekAudioParams {
    pub position_tick: u32,
}
