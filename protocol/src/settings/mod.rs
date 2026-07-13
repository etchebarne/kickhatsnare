mod get;
mod set;

pub use get::{GetSettings, GetSettingsParams};
pub use set::{SetSetting, SetSettingParams};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{ContractMethod, contract::describe};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct SettingsSnapshot {
    #[ts(inline)]
    pub categories: Vec<SettingCategory>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct SettingCategory {
    pub id: String,
    pub label: String,
    pub description: String,
    #[ts(inline)]
    pub settings: Vec<Setting>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Setting {
    IntegerSelect {
        id: String,
        label: String,
        description: String,
        value: u32,
        default_value: u32,
        unit: String,
        #[ts(inline)]
        options: Vec<IntegerSettingOption>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct IntegerSettingOption {
    pub value: u32,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema, TS)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
#[ts(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum SettingValue {
    Integer(u32),
}

pub(crate) fn methods() -> Vec<ContractMethod> {
    vec![describe::<GetSettings>(), describe::<SetSetting>()]
}
