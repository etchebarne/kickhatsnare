use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer};
use ts_rs::TS;

use super::{LoopRegion, TransportSnapshot};
use crate::IpcMethod;

pub struct SetLoopRegion;

impl IpcMethod for SetLoopRegion {
    const NAME: &'static str = "audio.setLoopRegion";
    type Params = SetLoopRegionParams;
    type Result = TransportSnapshot;
}

#[derive(Debug, JsonSchema, TS)]
#[schemars(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct SetLoopRegionParams {
    #[schemars(required, schema_with = "nullable_loop_region_schema")]
    #[ts(inline)]
    pub region: Option<LoopRegion>,
}

impl<'de> Deserialize<'de> for SetLoopRegionParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct WireParams {
            #[serde(deserialize_with = "deserialize_region")]
            region: Option<LoopRegion>,
        }

        let params = WireParams::deserialize(deserializer)?;
        Ok(Self {
            region: params.region,
        })
    }
}

fn deserialize_region<'de, D>(deserializer: D) -> Result<Option<LoopRegion>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::deserialize(deserializer)
}

fn nullable_loop_region_schema(generator: &mut SchemaGenerator) -> Schema {
    generator.subschema_for::<Option<LoopRegion>>()
}
