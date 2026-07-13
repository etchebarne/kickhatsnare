mod get;
mod pin_folder;
mod unpin_folder;

pub use get::{GetLibrary, GetLibraryParams};
pub use pin_folder::{PinFolder, PinFolderParams};
pub use unpin_folder::{UnpinFolder, UnpinFolderParams};

use schemars::JsonSchema;
use serde::Serialize;
use ts_rs::TS;

use crate::{ContractMethod, contract::describe};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct LibrarySnapshot {
    #[ts(inline)]
    pub pinned_folders: Vec<PinnedFolder>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct PinnedFolder {
    pub id: String,
    pub name: String,
    pub path: String,
    pub files: Vec<String>,
    pub is_available: bool,
}

pub(crate) fn methods() -> Vec<ContractMethod> {
    vec![
        describe::<GetLibrary>(),
        describe::<PinFolder>(),
        describe::<UnpinFolder>(),
    ]
}
