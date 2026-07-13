use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::TransportSnapshot;
use crate::IpcMethod;

pub struct StopAudio;

impl IpcMethod for StopAudio {
    const NAME: &'static str = "audio.stop";
    type Params = StopAudioParams;
    type Result = TransportSnapshot;
}

#[derive(Debug, Default, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(type = "Record<string, never>")]
pub struct StopAudioParams {}
