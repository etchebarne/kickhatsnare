use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::{ClipStretchMode, WorkspaceSnapshot};
use crate::IpcMethod;

pub struct SetTimelineClipProperties;

impl IpcMethod for SetTimelineClipProperties {
    const NAME: &'static str = "workspace.setTimelineClipProperties";
    type Params = SetTimelineClipPropertiesParams;
    type Result = WorkspaceSnapshot;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct SetTimelineClipPropertiesParams {
    pub id: String,
    #[ts(inline)]
    pub stretch_mode: ClipStretchMode,
    pub gain_db: f64,
    pub pan: f64,
    pub pitch_semitones: f64,
    pub tempo_percent: Option<f64>,
    pub make_unique: bool,
}
