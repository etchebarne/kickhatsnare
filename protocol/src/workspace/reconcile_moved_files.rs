use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct ReconcileMovedWorkspaceFiles;

impl IpcMethod for ReconcileMovedWorkspaceFiles {
    const NAME: &'static str = "workspace.reconcileMovedFiles";
    type Params = ReconcileMovedWorkspaceFilesParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct ReconcileMovedWorkspaceFilesParams {
    #[ts(inline)]
    pub moves: Vec<WorkspaceFileMove>,
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct WorkspaceFileMove {
    pub source_path: String,
    pub destination_path: String,
}
