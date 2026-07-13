use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::WorkspaceSnapshot;
use crate::IpcMethod;

pub struct ImportWorkspaceAudio;

impl IpcMethod for ImportWorkspaceAudio {
    const NAME: &'static str = "workspace.importAudio";
    type Params = ImportWorkspaceAudioParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct ImportWorkspaceAudioParams {
    pub source_paths: Vec<String>,
    pub target_directory: String,
}
