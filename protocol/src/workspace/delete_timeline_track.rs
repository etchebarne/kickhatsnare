use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct DeleteTimelineTrack;

impl IpcMethod for DeleteTimelineTrack {
    const NAME: &'static str = "workspace.deleteTimelineTrack";
    type Params = DeleteTimelineTrackParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct DeleteTimelineTrackParams {
    pub id: String,
}
