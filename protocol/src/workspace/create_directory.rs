use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct CreateWorkspaceDirectory;

impl IpcMethod for CreateWorkspaceDirectory {
    const NAME: &'static str = "workspace.createDirectory";
    type Params = CreateWorkspaceDirectoryParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct CreateWorkspaceDirectoryParams {
    pub path: String,
}
