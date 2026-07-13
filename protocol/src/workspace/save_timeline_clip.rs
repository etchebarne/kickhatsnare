use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct SaveTimelineClip;

impl IpcMethod for SaveTimelineClip {
    const NAME: &'static str = "workspace.saveTimelineClip";
    type Params = SaveTimelineClipParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct SaveTimelineClipParams {
    pub id: Option<String>,
    pub track_id: String,
    pub name: String,
    pub start_tick: u32,
    pub duration_ticks: u32,
    pub source_offset_ticks: u32,
}
