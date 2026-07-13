use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct MoveWorkspaceEntry;

impl IpcMethod for MoveWorkspaceEntry {
    const NAME: &'static str = "workspace.moveEntry";
    type Params = MoveWorkspaceEntryParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct MoveWorkspaceEntryParams {
    pub source_path: String,
    pub destination_path: String,
}
