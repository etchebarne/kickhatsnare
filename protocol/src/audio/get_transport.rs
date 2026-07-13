use schemars::JsonSchema;
use serde::Deserialize;
use ts_rs::TS;

use super::TransportSnapshot;
use crate::IpcMethod;

pub struct GetTransport;

impl IpcMethod for GetTransport {
    const NAME: &'static str = "audio.getTransport";
    type Params = GetTransportParams;
    type Result = TransportSnapshot;
}

#[derive(Debug, Default, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(type = "Record<string, never>")]
pub struct GetTransportParams {}
