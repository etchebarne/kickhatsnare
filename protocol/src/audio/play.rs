use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::TransportSnapshot;
use crate::IpcMethod;

pub struct PlayAudio;

impl IpcMethod for PlayAudio {
    const NAME: &'static str = "audio.play";
    type Params = PlayAudioParams;
    type Result = TransportSnapshot;
}

#[derive(Debug, Default, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(type = "Record<string, never>")]
pub struct PlayAudioParams {}
