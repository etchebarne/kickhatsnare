mod create_directory;
mod delete_entry;
mod get;
mod import_audio;
mod move_entry;
mod new;
mod open;
mod save;
mod save_as;

pub use create_directory::{CreateWorkspaceDirectory, CreateWorkspaceDirectoryParams};
pub use delete_entry::{DeleteWorkspaceEntry, DeleteWorkspaceEntryParams};
pub use get::{GetWorkspace, GetWorkspaceParams};
pub use import_audio::{ImportWorkspaceAudio, ImportWorkspaceAudioParams};
pub use move_entry::{MoveWorkspaceEntry, MoveWorkspaceEntryParams};
pub use new::{NewWorkspace, NewWorkspaceParams};
pub use open::{OpenWorkspace, OpenWorkspaceParams};
pub use save::{SaveWorkspace, SaveWorkspaceParams};
pub use save_as::{SaveWorkspaceAs, SaveWorkspaceAsParams};

use schemars::JsonSchema;
use serde::Serialize;
use ts_rs::TS;

use crate::{ContractMethod, contract::describe};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct WorkspaceSnapshot {
    pub name: String,
    pub root_path: Option<String>,
    pub project_file_path: Option<String>,
    pub files: Vec<String>,
    pub is_dirty: bool,
}

pub(crate) fn methods() -> Vec<ContractMethod> {
    vec![
        describe::<CreateWorkspaceDirectory>(),
        describe::<DeleteWorkspaceEntry>(),
        describe::<GetWorkspace>(),
        describe::<ImportWorkspaceAudio>(),
        describe::<MoveWorkspaceEntry>(),
        describe::<NewWorkspace>(),
        describe::<OpenWorkspace>(),
        describe::<SaveWorkspace>(),
        describe::<SaveWorkspaceAs>(),
    ]
}
