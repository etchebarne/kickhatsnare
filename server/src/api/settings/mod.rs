mod get;
mod set;

use kickhatsnare_core::{
    Core,
    settings::{Setting as CoreSetting, SettingsSnapshot as CoreSettingsSnapshot},
};
use kickhatsnare_protocol::{
    ErrorCode, IpcMethod,
    settings::{
        GetSettings, IntegerSettingOption, SetSetting, Setting, SettingCategory, SettingsSnapshot,
    },
};
use serde_json::Value;

use super::ApiError;

pub(super) fn dispatch(
    method: &str,
    action: &str,
    params: &Value,
    core: &mut Core,
) -> Result<Value, ApiError> {
    match method {
        GetSettings::NAME => get::handle(params, core),
        SetSetting::NAME => set::handle(params, core),
        _ => Err(ApiError::method_not_found("settings", action)),
    }
}

fn serialize_snapshot(snapshot: CoreSettingsSnapshot) -> Result<Value, ApiError> {
    let categories = snapshot
        .categories
        .into_iter()
        .map(|category| SettingCategory {
            id: category.id,
            label: category.label,
            description: category.description,
            settings: category
                .settings
                .into_iter()
                .map(|setting| match setting {
                    CoreSetting::IntegerSelect {
                        id,
                        label,
                        description,
                        value,
                        default_value,
                        unit,
                        options,
                    } => Setting::IntegerSelect {
                        id,
                        label,
                        description,
                        value,
                        default_value,
                        unit,
                        options: options
                            .into_iter()
                            .map(|option| IntegerSettingOption {
                                value: option.value,
                                label: option.label,
                            })
                            .collect(),
                    },
                })
                .collect(),
        })
        .collect();
    serde_json::to_value(SettingsSnapshot { categories })
        .map_err(|error| ApiError::new(ErrorCode::InternalError, error.to_string()))
}

fn core_error(error: &kickhatsnare_core::CoreError) -> ApiError {
    ApiError::new(ErrorCode::InternalError, error.to_string())
}
