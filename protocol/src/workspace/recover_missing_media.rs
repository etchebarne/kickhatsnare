use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct RecoverMissingWorkspaceMedia;

impl IpcMethod for RecoverMissingWorkspaceMedia {
    const NAME: &'static str = "workspace.recoverMissingMedia";
    type Params = RecoverMissingWorkspaceMediaParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct RecoverMissingWorkspaceMediaParams {
    pub source_path: String,
    #[ts(inline)]
    pub action: MissingMediaAction,
    pub replacement_path: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum MissingMediaAction {
    Replace,
    LeaveEmpty,
    DeleteClips,
}
