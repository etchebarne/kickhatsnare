use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::LibrarySnapshot;
use crate::IpcMethod;

pub struct GetLibrary;

impl IpcMethod for GetLibrary {
    const NAME: &'static str = "library.get";
    type Params = GetLibraryParams;
    type Result = LibrarySnapshot;
}

#[derive(Debug, Default, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(type = "Record<string, never>")]
pub struct GetLibraryParams {}
