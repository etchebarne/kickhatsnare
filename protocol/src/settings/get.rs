use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::SettingsSnapshot;
use crate::IpcMethod;

pub struct GetSettings;

impl IpcMethod for GetSettings {
    const NAME: &'static str = "settings.get";
    type Params = GetSettingsParams;
    type Result = SettingsSnapshot;
}

#[derive(Debug, Default, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(type = "Record<string, never>")]
pub struct GetSettingsParams {}
