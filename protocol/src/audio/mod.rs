mod get_transport;
mod pause;
mod play;
mod seek;
mod set_loop_region;
mod stop;

pub use get_transport::{GetTransport, GetTransportParams};
pub use pause::{PauseAudio, PauseAudioParams};
pub use play::{PlayAudio, PlayAudioParams};
pub use seek::{SeekAudio, SeekAudioParams};
pub use set_loop_region::{SetLoopRegion, SetLoopRegionParams};
pub use stop::{StopAudio, StopAudioParams};

use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{ContractMethod, contract::describe};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum TransportState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct LoopRegion {
    pub start_tick: u32,
    pub end_tick: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct TransportSnapshot {
    #[ts(inline)]
    pub state: TransportState,
    pub position_tick: u32,
    pub duration_ticks: u32,
    #[schemars(required, schema_with = "nullable_loop_region_schema")]
    #[ts(inline)]
    pub loop_region: Option<LoopRegion>,
    #[schemars(required, schema_with = "nullable_string_schema")]
    pub last_error: Option<String>,
}

fn nullable_loop_region_schema(generator: &mut SchemaGenerator) -> Schema {
    generator.subschema_for::<Option<LoopRegion>>()
}

fn nullable_string_schema(generator: &mut SchemaGenerator) -> Schema {
    generator.subschema_for::<Option<String>>()
}

pub(crate) fn methods() -> Vec<ContractMethod> {
    vec![
        describe::<GetTransport>(),
        describe::<PauseAudio>(),
        describe::<PlayAudio>(),
        describe::<SeekAudio>(),
        describe::<SetLoopRegion>(),
        describe::<StopAudio>(),
    ]
}
