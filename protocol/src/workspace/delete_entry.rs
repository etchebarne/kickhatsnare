use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct DeleteWorkspaceEntry;

impl IpcMethod for DeleteWorkspaceEntry {
    const NAME: &'static str = "workspace.deleteEntry";
    type Params = DeleteWorkspaceEntryParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct DeleteWorkspaceEntryParams {
    pub path: String,
}
