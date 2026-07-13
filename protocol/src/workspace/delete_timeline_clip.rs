use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct DeleteTimelineClip;

impl IpcMethod for DeleteTimelineClip {
    const NAME: &'static str = "workspace.deleteTimelineClip";
    type Params = DeleteTimelineClipParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct DeleteTimelineClipParams {
    pub id: String,
}
