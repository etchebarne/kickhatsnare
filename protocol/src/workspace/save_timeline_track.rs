use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct SaveTimelineTrack;

impl IpcMethod for SaveTimelineTrack {
    const NAME: &'static str = "workspace.saveTimelineTrack";
    type Params = SaveTimelineTrackParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct SaveTimelineTrackParams {
    pub id: Option<String>,
    pub name: String,
    pub is_muted: bool,
    pub is_soloed: bool,
    pub gain_db: f64,
    pub pan: f64,
    pub is_connected: bool,
}
