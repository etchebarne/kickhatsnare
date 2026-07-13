mod get;
mod pin_folder;
mod unpin_folder;

use kickhatsnare_core::library::Library;
use kickhatsnare_protocol::{
    IpcMethod,
    library::{GetLibrary, PinFolder, UnpinFolder},
};
use serde_json::Value;

use super::ApiError;

pub(super) fn dispatch(
    method: &str,
    action: &str,
    params: &Value,
    library: &mut Library,
) -> Result<Value, ApiError> {
    match method {
        GetLibrary::NAME => get::handle(params, library),
        PinFolder::NAME => pin_folder::handle(params, library),
        UnpinFolder::NAME => unpin_folder::handle(params, library),
        _ => Err(ApiError::method_not_found("library", action)),
    }
}

fn serialize_snapshot(
    snapshot: kickhatsnare_core::library::LibrarySnapshot,
) -> Result<Value, ApiError> {
    let pinned_folders = snapshot
        .pinned_folders
        .into_iter()
        .map(|folder| kickhatsnare_protocol::library::PinnedFolder {
            id: folder.id.to_string(),
            name: folder.name,
            path: folder.path.to_string_lossy().into_owned(),
            files: folder.files,
            is_available: folder.is_available,
        })
        .collect();
    serde_json::to_value(kickhatsnare_protocol::library::LibrarySnapshot { pinned_folders })
        .map_err(|error| {
            ApiError::new(
                kickhatsnare_protocol::ErrorCode::InternalError,
                error.to_string(),
            )
        })
}

fn core_error(error: &kickhatsnare_core::CoreError) -> ApiError {
    ApiError::new(
        kickhatsnare_protocol::ErrorCode::InternalError,
        error.to_string(),
    )
}
