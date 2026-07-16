use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct SplitTimelineClip;

impl IpcMethod for SplitTimelineClip {
    const NAME: &'static str = "workspace.splitTimelineClip";
    type Params = SplitTimelineClipParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct SplitTimelineClipParams {
    pub id: String,
    pub split_tick: u32,
}
