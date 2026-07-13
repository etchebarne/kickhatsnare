use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct OpenWorkspace;

impl IpcMethod for OpenWorkspace {
    const NAME: &'static str = "workspace.open";
    type Params = OpenWorkspaceParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct OpenWorkspaceParams {
    pub project_file_path: String,
}
