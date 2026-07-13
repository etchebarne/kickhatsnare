use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::{GridDivision, WorkspaceSnapshot};
use crate::IpcMethod;

pub struct SetTimelineSettings;

impl IpcMethod for SetTimelineSettings {
    const NAME: &'static str = "workspace.setTimelineSettings";
    type Params = SetTimelineSettingsParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct SetTimelineSettingsParams {
    pub bpm: f64,
    pub time_signature_numerator: u8,
    pub time_signature_denominator: u8,
    #[ts(inline)]
    pub grid_division: GridDivision,
    pub is_snap_enabled: bool,
}
