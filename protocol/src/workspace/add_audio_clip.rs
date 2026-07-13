use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct AddAudioClip;

impl IpcMethod for AddAudioClip {
    const NAME: &'static str = "workspace.addAudioClip";
    type Params = AddAudioClipParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct AddAudioClipParams {
    pub track_id: String,
    pub source_path: String,
    pub start_tick: u32,
}
