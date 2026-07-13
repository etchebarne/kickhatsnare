use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct UndoWorkspace;

impl IpcMethod for UndoWorkspace {
    const NAME: &'static str = "workspace.undo";
    type Params = UndoWorkspaceParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Default, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(type = "Record<string, never>")]
pub struct UndoWorkspaceParams {}
