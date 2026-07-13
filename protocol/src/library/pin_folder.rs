use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::LibrarySnapshot;
use crate::IpcMethod;

pub struct PinFolder;

impl IpcMethod for PinFolder {
    const NAME: &'static str = "library.pinFolder";
    type Params = PinFolderParams;
    type Result = LibrarySnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct PinFolderParams {
    pub path: String,
}
