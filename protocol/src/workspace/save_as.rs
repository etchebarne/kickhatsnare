use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct SaveWorkspaceAs;

impl IpcMethod for SaveWorkspaceAs {
    const NAME: &'static str = "workspace.saveAs";
    type Params = SaveWorkspaceAsParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct SaveWorkspaceAsParams {
    pub directory_path: String,
}
