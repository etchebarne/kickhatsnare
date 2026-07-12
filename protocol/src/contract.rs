use schemars::JsonSchema;
use serde_json::Value;
use ts_rs::{Config, TS};

use crate::{PROTOCOL_VERSION, audio, system, workspace};

pub trait IpcMethod {
    const NAME: &'static str;
    type Params: JsonSchema + TS;
    type Result: JsonSchema + TS;
}

pub struct Contract {
    pub version: u32,
    pub methods: Vec<ContractMethod>,
}

pub struct ContractMethod {
    pub name: &'static str,
    pub params_type: String,
    pub result_type: String,
    pub params_schema: Value,
    pub result_schema: Value,
}

#[must_use]
pub fn contract() -> Contract {
    let mut methods = Vec::new();
    methods.extend(system::methods());
    methods.extend(audio::methods());
    methods.extend(workspace::methods());

    Contract {
        version: PROTOCOL_VERSION,
        methods,
    }
}

pub(crate) fn describe<M: IpcMethod>() -> ContractMethod {
    let typescript = Config::default();

    ContractMethod {
        name: M::NAME,
        params_type: M::Params::inline(&typescript),
        result_type: M::Result::inline(&typescript),
        params_schema: schema::<M::Params>(),
        result_schema: schema::<M::Result>(),
    }
}

fn schema<T: JsonSchema>() -> Value {
    serde_json::to_value(schemars::schema_for!(T)).expect("JSON Schema should be serializable")
}
