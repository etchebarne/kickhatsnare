use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::{SettingValue, SettingsSnapshot};
use crate::IpcMethod;

pub struct SetSetting;

impl IpcMethod for SetSetting {
    const NAME: &'static str = "settings.set";
    type Params = SetSettingParams;
    type Result = SettingsSnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct SetSettingParams {
    pub id: String,
    #[ts(inline)]
    pub value: SettingValue,
}
