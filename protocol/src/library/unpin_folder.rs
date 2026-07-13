use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::LibrarySnapshot;
use crate::IpcMethod;

pub struct UnpinFolder;

impl IpcMethod for UnpinFolder {
    const NAME: &'static str = "library.unpinFolder";
    type Params = UnpinFolderParams;
    type Result = LibrarySnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct UnpinFolderParams {
    pub id: String,
}
