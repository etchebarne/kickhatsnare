use std::{
    collections::HashSet,
    fs::{self, File},
    io::{self, Write},
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use crate::CoreError;

mod timeline;

use timeline::Timeline;
pub use timeline::{GridDivision, TimelineClipSnapshot, TimelineSnapshot, TimelineTrackSnapshot};

const PROJECT_FILE_NAME: &str = "project.khs";
const PROJECT_FORMAT_VERSION: u32 = 2;
const OLDEST_SUPPORTED_PROJECT_FORMAT_VERSION: u32 = 1;

/// Owns the active project session and its persistence operations.
#[derive(Debug)]
pub struct Workspaces {
    active: Workspace,
}

#[derive(Debug)]
struct Workspace {
    name: String,
    root_path: Option<PathBuf>,
    project_file_path: Option<PathBuf>,
    pending_imports: Vec<PendingImport>,
    virtual_directories: HashSet<PathBuf>,
    timeline: Timeline,
    is_dirty: bool,
}

#[derive(Debug, Clone)]
struct PendingImport {
    source_path: PathBuf,
    target_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceSnapshot {
    pub name: String,
    pub root_path: Option<PathBuf>,
    pub project_file_path: Option<PathBuf>,
    pub files: Vec<String>,
    pub timeline: TimelineSnapshot,
    pub is_dirty: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectDocument {
    format_version: u32,
    name: String,
    #[serde(default)]
    timeline: Timeline,
}

impl Default for Workspaces {
    fn default() -> Self {
        Self {
            active: Workspace::new(),
        }
    }
}

impl Workspace {
    fn new() -> Self {
        Self {
            name: "Untitled".to_owned(),
            root_path: None,
            project_file_path: None,
            pending_imports: Vec::new(),
            virtual_directories: HashSet::new(),
            timeline: Timeline::default(),
            is_dirty: false,
        }
    }
}

impl Workspaces {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces the active session with a new unsaved project.
    ///
    /// # Errors
    ///
    /// Returns an error if the resulting workspace snapshot cannot be created.
    pub fn new_project(&mut self) -> Result<WorkspaceSnapshot, CoreError> {
        self.active = Workspace::new();
        self.snapshot()
    }

    /// Returns the current project state and its visible workspace files.
    ///
    /// # Errors
    ///
    /// Returns an error if a saved project directory cannot be read.
    pub fn snapshot(&self) -> Result<WorkspaceSnapshot, CoreError> {
        let files = if let Some(root_path) = &self.active.root_path {
            collect_workspace_files(root_path, self.active.project_file_path.as_deref())?
        } else {
            let mut files = self
                .active
                .virtual_directories
                .iter()
                .map(|directory| format!("{}/", relative_path_string(directory)))
                .chain(
                    self.active
                        .pending_imports
                        .iter()
                        .map(|pending| relative_path_string(&pending.target_path)),
                )
                .collect::<Vec<_>>();
            files.sort();
            files
        };

        Ok(WorkspaceSnapshot {
            name: self.active.name.clone(),
            root_path: self.active.root_path.clone(),
            project_file_path: self.active.project_file_path.clone(),
            files,
            timeline: self.active.timeline.snapshot(),
            is_dirty: self.active.is_dirty,
        })
    }

    /// Materializes the active project in a new workspace directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory is invalid or the project cannot be written.
    pub fn save_as(&mut self, root_path: impl AsRef<Path>) -> Result<WorkspaceSnapshot, CoreError> {
        let root_path = root_path.as_ref();
        let name = project_name_from_path(root_path)?;
        let project_file_path = root_path.join(PROJECT_FILE_NAME);

        fs::create_dir_all(root_path).map_err(|error| {
            CoreError::new(format!(
                "failed to create project directory {}: {error}",
                root_path.display()
            ))
        })?;
        let mut directories = self.active.virtual_directories.iter().collect::<Vec<_>>();
        directories.sort_by_key(|directory| directory.components().count());
        for directory in directories {
            fs::create_dir_all(root_path.join(directory)).map_err(|error| {
                CoreError::new(format!(
                    "failed to create project directory {}: {error}",
                    directory.display()
                ))
            })?;
        }
        for pending in &self.active.pending_imports {
            copy_audio_file(&pending.source_path, &root_path.join(&pending.target_path))?;
        }
        write_project(&project_file_path, &name, &self.active.timeline)?;

        self.active = Workspace {
            name,
            root_path: Some(root_path.to_owned()),
            project_file_path: Some(project_file_path),
            pending_imports: Vec::new(),
            virtual_directories: HashSet::new(),
            timeline: self.active.timeline.clone(),
            is_dirty: false,
        };
        self.snapshot()
    }

    /// Saves the active project to its existing project file.
    ///
    /// # Errors
    ///
    /// Returns an error if the project is unsaved or its project file cannot be written.
    pub fn save(&mut self) -> Result<WorkspaceSnapshot, CoreError> {
        let project_file_path = self
            .active
            .project_file_path
            .as_ref()
            .ok_or_else(|| CoreError::new("project has not been saved yet"))?;

        write_project(project_file_path, &self.active.name, &self.active.timeline)?;
        self.active.is_dirty = false;
        self.snapshot()
    }

    /// Opens a project file and makes its containing directory the active workspace.
    ///
    /// # Errors
    ///
    /// Returns an error if the project cannot be read, parsed, or uses an unsupported format.
    pub fn open(
        &mut self,
        project_file_path: impl AsRef<Path>,
    ) -> Result<WorkspaceSnapshot, CoreError> {
        let project_file_path = project_file_path.as_ref();
        let contents = fs::read(project_file_path).map_err(|error| {
            CoreError::new(format!(
                "failed to read project file {}: {error}",
                project_file_path.display()
            ))
        })?;
        let mut document: ProjectDocument = serde_json::from_slice(&contents).map_err(|error| {
            CoreError::new(format!(
                "failed to parse project file {}: {error}",
                project_file_path.display()
            ))
        })?;

        if !(OLDEST_SUPPORTED_PROJECT_FORMAT_VERSION..=PROJECT_FORMAT_VERSION)
            .contains(&document.format_version)
        {
            return Err(CoreError::new(format!(
                "unsupported project format version {}; expected {OLDEST_SUPPORTED_PROJECT_FORMAT_VERSION} through {PROJECT_FORMAT_VERSION}",
                document.format_version
            )));
        }
        document.timeline.ensure_minimum_track()?;
        document.timeline.validate()?;

        let root_path = project_file_path
            .parent()
            .ok_or_else(|| CoreError::new("project file must have a parent directory"))?;
        self.active = Workspace {
            name: document.name,
            root_path: Some(root_path.to_owned()),
            project_file_path: Some(project_file_path.to_owned()),
            pending_imports: Vec::new(),
            virtual_directories: HashSet::new(),
            timeline: document.timeline,
            is_dirty: false,
        };
        self.snapshot()
    }

    /// Imports audio files into the active workspace.
    ///
    /// Unsaved projects retain source paths in memory and materialize the files
    /// during the first save. Saved projects copy files immediately.
    ///
    /// # Errors
    ///
    /// Returns an error if a source is invalid, unsupported, or cannot be copied.
    pub fn import_audio_files(
        &mut self,
        source_paths: impl IntoIterator<Item = PathBuf>,
        target_directory: impl AsRef<Path>,
    ) -> Result<WorkspaceSnapshot, CoreError> {
        let target_directory = validate_target_directory(target_directory.as_ref())?;
        let root_path = self.active.root_path.clone();
        let destination_directory = root_path
            .as_deref()
            .map(|root_path| resolve_workspace_directory(root_path, &target_directory))
            .transpose()?;
        if root_path.is_none()
            && !target_directory.as_os_str().is_empty()
            && !self.active.virtual_directories.contains(&target_directory)
        {
            return Err(CoreError::new(
                "unsaved projects can only import audio into the project root",
            ));
        }

        let mut reserved_names = workspace_directory_names(destination_directory.as_deref())?;
        for pending in &self.active.pending_imports {
            if pending.target_path.parent() == Some(target_directory.as_path())
                && let Some(name) = pending
                    .target_path
                    .file_name()
                    .and_then(|name| name.to_str())
            {
                reserved_names.insert(name.to_owned());
            }
        }
        for directory in &self.active.virtual_directories {
            if directory.parent() == Some(target_directory.as_path())
                && let Some(name) = directory.file_name().and_then(|name| name.to_str())
            {
                reserved_names.insert(name.to_owned());
            }
        }

        let mut imports = Vec::new();
        for source_path in source_paths {
            let source_path = validate_audio_source(&source_path)?;
            if destination_directory
                .as_ref()
                .is_some_and(|directory| source_path.parent() == Some(directory.as_path()))
            {
                continue;
            }

            let target_name = reserve_target_name(&source_path, &mut reserved_names)?;
            imports.push(PendingImport {
                source_path,
                target_path: target_directory.join(target_name),
            });
        }

        if let Some(destination_directory) = destination_directory {
            for pending in imports {
                let target_name = pending
                    .target_path
                    .file_name()
                    .ok_or_else(|| CoreError::new("audio import target must have a file name"))?;
                copy_audio_file(
                    &pending.source_path,
                    &destination_directory.join(target_name),
                )?;
            }
        } else if !imports.is_empty() {
            self.active.pending_imports.extend(imports);
            self.active.is_dirty = true;
        }

        self.snapshot()
    }

    /// Creates a directory at a workspace-relative path.
    ///
    /// # Errors
    ///
    /// Returns an error if the path is invalid, its parent is missing, or it already exists.
    pub fn create_directory(
        &mut self,
        relative_path: impl AsRef<Path>,
    ) -> Result<WorkspaceSnapshot, CoreError> {
        let relative_path = validate_entry_path(relative_path.as_ref())?;
        let parent = relative_path.parent().unwrap_or_else(|| Path::new(""));

        if let Some(root_path) = self.active.root_path.as_deref() {
            let parent = resolve_workspace_directory(root_path, parent)?;
            let name = relative_path
                .file_name()
                .ok_or_else(|| CoreError::new("directory must have a name"))?;
            let destination = parent.join(name);
            if destination.exists() {
                return Err(CoreError::new("workspace entry already exists"));
            }
            fs::create_dir(&destination).map_err(|error| {
                CoreError::new(format!(
                    "failed to create directory {}: {error}",
                    destination.display()
                ))
            })?;
        } else {
            if !parent.as_os_str().is_empty() && !self.active.virtual_directories.contains(parent) {
                return Err(CoreError::new("directory parent does not exist"));
            }
            if self.active.virtual_directories.contains(&relative_path)
                || self
                    .active
                    .pending_imports
                    .iter()
                    .any(|pending| pending.target_path == relative_path)
            {
                return Err(CoreError::new("workspace entry already exists"));
            }
            self.active.virtual_directories.insert(relative_path);
            self.active.is_dirty = true;
        }

        self.snapshot()
    }

    /// Deletes a file or directory at a workspace-relative path.
    ///
    /// # Errors
    ///
    /// Returns an error if the path is invalid, missing, or cannot be removed.
    pub fn delete_entry(
        &mut self,
        relative_path: impl AsRef<Path>,
    ) -> Result<WorkspaceSnapshot, CoreError> {
        let relative_path = validate_entry_path(relative_path.as_ref())?;

        if let Some(root_path) = self.active.root_path.as_deref() {
            let target = workspace_entry_path(root_path, &relative_path)?;
            if self.active.project_file_path.as_deref() == Some(target.as_path()) {
                return Err(CoreError::new("the project file cannot be deleted"));
            }
            let metadata = fs::symlink_metadata(&target).map_err(|error| {
                CoreError::new(format!(
                    "failed to inspect workspace entry {}: {error}",
                    target.display()
                ))
            })?;
            if metadata.is_dir() && !metadata.file_type().is_symlink() {
                fs::remove_dir_all(&target)
            } else {
                fs::remove_file(&target)
            }
            .map_err(|error| {
                CoreError::new(format!(
                    "failed to delete workspace entry {}: {error}",
                    target.display()
                ))
            })?;
        } else if self.active.virtual_directories.contains(&relative_path) {
            self.active
                .virtual_directories
                .retain(|directory| !directory.starts_with(&relative_path));
            self.active
                .pending_imports
                .retain(|pending| !pending.target_path.starts_with(&relative_path));
            self.active.is_dirty = true;
        } else if let Some(index) = self
            .active
            .pending_imports
            .iter()
            .position(|pending| pending.target_path == relative_path)
        {
            self.active.pending_imports.remove(index);
            self.active.is_dirty = true;
        } else {
            return Err(CoreError::new("workspace entry does not exist"));
        }

        self.snapshot()
    }

    /// Moves or renames a workspace entry.
    ///
    /// # Errors
    ///
    /// Returns an error if either path is invalid or the entry cannot be moved.
    pub fn move_entry(
        &mut self,
        source_path: impl AsRef<Path>,
        destination_path: impl AsRef<Path>,
    ) -> Result<WorkspaceSnapshot, CoreError> {
        let source_path = validate_entry_path(source_path.as_ref())?;
        let destination_path = validate_entry_path(destination_path.as_ref())?;
        if source_path == destination_path {
            return self.snapshot();
        }

        if let Some(root_path) = self.active.root_path.as_deref() {
            let source = workspace_entry_path(root_path, &source_path)?;
            if self.active.project_file_path.as_deref() == Some(source.as_path()) {
                return Err(CoreError::new("the project file cannot be moved"));
            }
            let source_metadata = fs::symlink_metadata(&source).map_err(|error| {
                CoreError::new(format!(
                    "failed to inspect workspace entry {}: {error}",
                    source.display()
                ))
            })?;
            if source_metadata.is_dir() && destination_path.starts_with(&source_path) {
                return Err(CoreError::new("a directory cannot be moved inside itself"));
            }
            let destination_parent = destination_path.parent().unwrap_or_else(|| Path::new(""));
            let destination_parent = resolve_workspace_directory(root_path, destination_parent)?;
            let destination_name = destination_path
                .file_name()
                .ok_or_else(|| CoreError::new("destination must have a name"))?;
            let destination = destination_parent.join(destination_name);
            if destination.exists() {
                return Err(CoreError::new("workspace destination already exists"));
            }
            fs::rename(&source, &destination).map_err(|error| {
                CoreError::new(format!(
                    "failed to move workspace entry {}: {error}",
                    source.display()
                ))
            })?;
        } else {
            move_unsaved_entry(&mut self.active, &source_path, &destination_path)?;
            self.active.is_dirty = true;
        }

        self.snapshot()
    }

    /// Updates the active project's musical timing and grid settings.
    ///
    /// # Errors
    ///
    /// Returns an error if the tempo or time signature is outside supported bounds.
    pub fn set_timeline_settings(
        &mut self,
        bpm: f64,
        time_signature_numerator: u8,
        time_signature_denominator: u8,
        grid_division: GridDivision,
        is_snap_enabled: bool,
    ) -> Result<WorkspaceSnapshot, CoreError> {
        self.active.timeline.set_settings(
            bpm,
            time_signature_numerator,
            time_signature_denominator,
            grid_division,
            is_snap_enabled,
        )?;
        self.active.is_dirty = true;
        self.snapshot()
    }

    /// Creates a timeline track or updates an existing one.
    ///
    /// # Errors
    ///
    /// Returns an error if the track does not exist or its name is invalid.
    pub fn save_timeline_track(
        &mut self,
        id: Option<&str>,
        name: &str,
        is_muted: bool,
        is_soloed: bool,
    ) -> Result<WorkspaceSnapshot, CoreError> {
        self.active
            .timeline
            .save_track(id, name, is_muted, is_soloed)?;
        self.active.is_dirty = true;
        self.snapshot()
    }

    /// Deletes a timeline track and its clips.
    ///
    /// # Errors
    ///
    /// Returns an error if the track does not exist.
    pub fn delete_timeline_track(&mut self, id: &str) -> Result<WorkspaceSnapshot, CoreError> {
        self.active.timeline.delete_track(id)?;
        self.active.is_dirty = true;
        self.snapshot()
    }

    /// Creates a timeline clip or updates and moves an existing one.
    ///
    /// # Errors
    ///
    /// Returns an error if the track or clip is missing or the clip range is invalid.
    #[allow(clippy::too_many_arguments)]
    pub fn save_timeline_clip(
        &mut self,
        id: Option<&str>,
        track_id: &str,
        name: &str,
        start_tick: u32,
        duration_ticks: u32,
        source_offset_ticks: u32,
    ) -> Result<WorkspaceSnapshot, CoreError> {
        self.active.timeline.save_clip(
            id,
            track_id,
            name,
            start_tick,
            duration_ticks,
            source_offset_ticks,
        )?;
        self.active.is_dirty = true;
        self.snapshot()
    }

    /// Deletes a timeline clip.
    ///
    /// # Errors
    ///
    /// Returns an error if the clip does not exist.
    pub fn delete_timeline_clip(&mut self, id: &str) -> Result<WorkspaceSnapshot, CoreError> {
        self.active.timeline.delete_clip(id)?;
        self.active.is_dirty = true;
        self.snapshot()
    }
}

fn validate_entry_path(path: &Path) -> Result<PathBuf, CoreError> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(CoreError::new("workspace entry must be a relative path"));
    }
    Ok(path.to_owned())
}

fn workspace_entry_path(root_path: &Path, relative_path: &Path) -> Result<PathBuf, CoreError> {
    let parent = relative_path.parent().unwrap_or_else(|| Path::new(""));
    let parent = resolve_workspace_directory(root_path, parent)?;
    let name = relative_path
        .file_name()
        .ok_or_else(|| CoreError::new("workspace entry must have a name"))?;
    Ok(parent.join(name))
}

fn move_unsaved_entry(
    workspace: &mut Workspace,
    source_path: &Path,
    destination_path: &Path,
) -> Result<(), CoreError> {
    let destination_parent = destination_path.parent().unwrap_or_else(|| Path::new(""));
    if !destination_parent.as_os_str().is_empty()
        && !workspace.virtual_directories.contains(destination_parent)
    {
        return Err(CoreError::new(
            "workspace destination parent does not exist",
        ));
    }
    if workspace.virtual_directories.contains(destination_path)
        || workspace
            .pending_imports
            .iter()
            .any(|pending| pending.target_path == destination_path)
    {
        return Err(CoreError::new("workspace destination already exists"));
    }

    if workspace.virtual_directories.contains(source_path) {
        if destination_path.starts_with(source_path) {
            return Err(CoreError::new("a directory cannot be moved inside itself"));
        }
        let moved_directories = workspace
            .virtual_directories
            .iter()
            .filter(|directory| directory.starts_with(source_path))
            .cloned()
            .collect::<Vec<_>>();
        for directory in &moved_directories {
            workspace.virtual_directories.remove(directory);
        }
        for directory in moved_directories {
            let suffix = directory
                .strip_prefix(source_path)
                .map_err(|error| CoreError::new(format!("failed to move directory: {error}")))?;
            let moved_path = if suffix.as_os_str().is_empty() {
                destination_path.to_owned()
            } else {
                destination_path.join(suffix)
            };
            workspace.virtual_directories.insert(moved_path);
        }
        for pending in &mut workspace.pending_imports {
            if pending.target_path.starts_with(source_path) {
                let suffix = pending
                    .target_path
                    .strip_prefix(source_path)
                    .map_err(|error| {
                        CoreError::new(format!("failed to move imported file: {error}"))
                    })?;
                pending.target_path = destination_path.join(suffix);
            }
        }
        return Ok(());
    }

    let pending = workspace
        .pending_imports
        .iter_mut()
        .find(|pending| pending.target_path == source_path)
        .ok_or_else(|| CoreError::new("workspace entry does not exist"))?;
    destination_path.clone_into(&mut pending.target_path);
    Ok(())
}

fn validate_target_directory(target_directory: &Path) -> Result<PathBuf, CoreError> {
    if target_directory.is_absolute()
        || target_directory
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(CoreError::new(
            "audio import target must be a relative project directory",
        ));
    }

    Ok(target_directory.to_owned())
}

fn resolve_workspace_directory(
    root_path: &Path,
    target_directory: &Path,
) -> Result<PathBuf, CoreError> {
    let canonical_root = fs::canonicalize(root_path)
        .map_err(|error| CoreError::new(format!("failed to resolve project directory: {error}")))?;
    let directory = fs::canonicalize(root_path.join(target_directory)).map_err(|error| {
        CoreError::new(format!(
            "failed to resolve audio import directory {}: {error}",
            target_directory.display()
        ))
    })?;
    if !directory.starts_with(&canonical_root) || !directory.is_dir() {
        return Err(CoreError::new(
            "audio import target must be a directory inside the project",
        ));
    }

    Ok(directory)
}

fn validate_audio_source(source_path: &Path) -> Result<PathBuf, CoreError> {
    let source_path = fs::canonicalize(source_path).map_err(|error| {
        CoreError::new(format!(
            "failed to resolve audio file {}: {error}",
            source_path.display()
        ))
    })?;
    if !source_path.is_file() {
        return Err(CoreError::new(format!(
            "audio import source is not a file: {}",
            source_path.display()
        )));
    }

    let extension = source_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| CoreError::new("audio import source must have a supported extension"))?;
    if !matches!(
        extension.as_str(),
        "aif" | "aiff" | "flac" | "m4a" | "mp3" | "oga" | "ogg" | "opus" | "wav" | "wave"
    ) {
        return Err(CoreError::new(format!(
            "unsupported audio file extension: .{extension}"
        )));
    }

    Ok(source_path)
}

fn workspace_directory_names(directory: Option<&Path>) -> Result<HashSet<String>, CoreError> {
    let Some(directory) = directory else {
        return Ok(HashSet::new());
    };
    let entries = fs::read_dir(directory).map_err(|error| {
        CoreError::new(format!(
            "failed to read project directory {}: {error}",
            directory.display()
        ))
    })?;

    let mut names = HashSet::new();
    for entry in entries {
        let entry = entry
            .map_err(|error| CoreError::new(format!("failed to read project entry: {error}")))?;
        if let Some(name) = entry.file_name().to_str() {
            names.insert(name.to_owned());
        }
    }
    Ok(names)
}

fn relative_path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn reserve_target_name(
    source_path: &Path,
    reserved_names: &mut HashSet<String>,
) -> Result<String, CoreError> {
    let file_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| CoreError::new("audio file must have a valid UTF-8 name"))?;
    if reserved_names.insert(file_name.to_owned()) {
        return Ok(file_name.to_owned());
    }

    let stem = source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| CoreError::new("audio file must have a valid UTF-8 name"))?;
    let extension = source_path
        .extension()
        .and_then(|extension| extension.to_str())
        .ok_or_else(|| CoreError::new("audio file must have a valid UTF-8 extension"))?;
    for suffix in 2_u32.. {
        let candidate = format!("{stem} {suffix}.{extension}");
        if reserved_names.insert(candidate.clone()) {
            return Ok(candidate);
        }
    }

    unreachable!("file name suffix space is finite but cannot be exhausted in practice")
}

fn copy_audio_file(source_path: &Path, destination_path: &Path) -> Result<(), CoreError> {
    let parent = destination_path
        .parent()
        .ok_or_else(|| CoreError::new("audio destination must have a parent directory"))?;
    let mut source = File::open(source_path).map_err(|error| {
        CoreError::new(format!(
            "failed to open audio file {}: {error}",
            source_path.display()
        ))
    })?;
    let mut temporary = NamedTempFile::new_in(parent).map_err(|error| {
        CoreError::new(format!(
            "failed to stage audio file in {}: {error}",
            parent.display()
        ))
    })?;
    io::copy(&mut source, &mut temporary)
        .and_then(|_| temporary.as_file().sync_all())
        .map_err(|error| {
            CoreError::new(format!(
                "failed to copy audio file {}: {error}",
                source_path.display()
            ))
        })?;
    temporary
        .persist_noclobber(destination_path)
        .map_err(|error| {
            CoreError::new(format!(
                "failed to store audio file {}: {error}",
                destination_path.display()
            ))
        })?;
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| {
            CoreError::new(format!(
                "failed to sync project directory {}: {error}",
                parent.display()
            ))
        })?;

    Ok(())
}

fn project_name_from_path(root_path: &Path) -> Result<String, CoreError> {
    root_path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| CoreError::new("project directory must have a valid UTF-8 name"))
}

fn write_project(
    project_file_path: &Path,
    name: &str,
    timeline: &Timeline,
) -> Result<(), CoreError> {
    let document = ProjectDocument {
        format_version: PROJECT_FORMAT_VERSION,
        name: name.to_owned(),
        timeline: timeline.clone(),
    };
    let contents = serde_json::to_vec_pretty(&document)
        .map_err(|error| CoreError::new(format!("failed to serialize project: {error}")))?;
    let parent = project_file_path
        .parent()
        .ok_or_else(|| CoreError::new("project file must have a parent directory"))?;
    let mut file = NamedTempFile::new_in(parent).map_err(|error| {
        CoreError::new(format!(
            "failed to create a temporary project file in {}: {error}",
            parent.display()
        ))
    })?;
    file.write_all(&contents)
        .and_then(|()| file.write_all(b"\n"))
        .and_then(|()| file.as_file().sync_all())
        .map_err(|error| {
            CoreError::new(format!(
                "failed to write a temporary project file in {}: {error}",
                parent.display()
            ))
        })?;
    file.persist(project_file_path).map_err(|error| {
        CoreError::new(format!(
            "failed to replace project file {}: {error}",
            project_file_path.display()
        ))
    })?;
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| {
            CoreError::new(format!(
                "failed to sync project directory {}: {error}",
                parent.display()
            ))
        })?;

    Ok(())
}

fn collect_workspace_files(
    root_path: &Path,
    project_file_path: Option<&Path>,
) -> Result<Vec<String>, CoreError> {
    let mut files = Vec::new();
    collect_files(root_path, root_path, project_file_path, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files(
    root_path: &Path,
    directory: &Path,
    project_file_path: Option<&Path>,
    files: &mut Vec<String>,
) -> Result<(), CoreError> {
    let entries = fs::read_dir(directory).map_err(|error| {
        CoreError::new(format!(
            "failed to read project directory {}: {error}",
            directory.display()
        ))
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            CoreError::new(format!(
                "failed to read an entry in project directory {}: {error}",
                directory.display()
            ))
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| {
            CoreError::new(format!(
                "failed to inspect project entry {}: {error}",
                path.display()
            ))
        })?;

        if file_type.is_dir() {
            let relative_path = path.strip_prefix(root_path).map_err(|error| {
                CoreError::new(format!("failed to resolve project directory: {error}"))
            })?;
            files.push(format!("{}/", relative_path_string(relative_path)));
            collect_files(root_path, &path, project_file_path, files)?;
        } else if project_file_path != Some(path.as_path()) {
            let relative_path = path.strip_prefix(root_path).map_err(|error| {
                CoreError::new(format!("failed to resolve project entry: {error}"))
            })?;
            files.push(relative_path.to_string_lossy().replace('\\', "/"));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::Value;
    use tempfile::tempdir;

    use super::{GridDivision, PROJECT_FILE_NAME, Workspaces};

    #[test]
    fn starts_with_an_empty_unsaved_project_tree() {
        let snapshot = Workspaces::new()
            .snapshot()
            .expect("snapshot should succeed");

        assert_eq!(snapshot.name, "Untitled");
        assert_eq!(snapshot.root_path, None);
        assert_eq!(snapshot.project_file_path, None);
        assert!(snapshot.files.is_empty());
        assert!((snapshot.timeline.bpm - 120.0).abs() < f64::EPSILON);
        assert_eq!(snapshot.timeline.tracks.len(), 30);
        assert!(!snapshot.is_dirty);
    }

    #[test]
    fn save_as_materializes_a_versioned_project() {
        let parent = tempdir().expect("temporary directory should be created");
        let root = parent.path().join("First Beat");
        let mut workspaces = Workspaces::new();

        let snapshot = workspaces.save_as(&root).expect("project should save");
        let project_file = root.join(PROJECT_FILE_NAME);

        assert_eq!(snapshot.name, "First Beat");
        assert_eq!(snapshot.root_path.as_deref(), Some(root.as_path()));
        assert_eq!(
            snapshot.project_file_path.as_deref(),
            Some(project_file.as_path())
        );
        assert!(snapshot.files.is_empty());
        let project: Value = serde_json::from_str(
            &fs::read_to_string(project_file).expect("project file should be readable"),
        )
        .expect("project should contain JSON");
        assert_eq!(project["formatVersion"], 2);
        assert_eq!(project["name"], "First Beat");
        assert_eq!(project["timeline"]["bpm"], 120.0);
    }

    #[test]
    fn open_restores_the_project_and_lists_workspace_files() {
        let parent = tempdir().expect("temporary directory should be created");
        let root = parent.path().join("First Beat");
        let mut workspaces = Workspaces::new();
        let saved = workspaces.save_as(&root).expect("project should save");
        fs::create_dir(root.join("Samples")).expect("sample directory should be created");
        fs::write(root.join("Samples/Kick.wav"), []).expect("sample should be created");

        let mut reopened = Workspaces::new();
        let snapshot = reopened
            .open(saved.project_file_path.expect("project should have a path"))
            .expect("project should open");

        assert_eq!(snapshot.name, "First Beat");
        assert_eq!(snapshot.files, ["Samples/", "Samples/Kick.wav"]);
    }

    #[test]
    fn rejects_an_unsupported_project_version_without_replacing_the_session() {
        let parent = tempdir().expect("temporary directory should be created");
        let project_file = parent.path().join(PROJECT_FILE_NAME);
        fs::write(&project_file, r#"{"formatVersion":3,"name":"Future"}"#)
            .expect("project should be created");
        let mut workspaces = Workspaces::new();

        let error = workspaces
            .open(project_file)
            .expect_err("project should be rejected");

        assert_eq!(
            error.to_string(),
            "unsupported project format version 3; expected 1 through 2"
        );
        assert_eq!(
            workspaces.snapshot().expect("snapshot should succeed").name,
            "Untitled"
        );
    }

    #[test]
    fn opens_version_one_projects_with_an_empty_default_timeline() {
        let parent = tempdir().expect("temporary directory should be created");
        let project_file = parent.path().join(PROJECT_FILE_NAME);
        fs::write(&project_file, r#"{"formatVersion":1,"name":"Legacy"}"#)
            .expect("project should be created");
        let mut workspaces = Workspaces::new();

        let snapshot = workspaces
            .open(project_file)
            .expect("version one project should migrate");

        assert_eq!(snapshot.name, "Legacy");
        assert!((snapshot.timeline.bpm - 120.0).abs() < f64::EPSILON);
        assert_eq!(snapshot.timeline.tracks.len(), 30);
    }

    #[test]
    fn restores_the_minimum_track_when_opening_an_empty_timeline() {
        let parent = tempdir().expect("temporary directory should be created");
        let project_file = parent.path().join(PROJECT_FILE_NAME);
        fs::write(
            &project_file,
            r#"{
                "formatVersion": 2,
                "name": "Empty",
                "timeline": {
                    "bpm": 120.0,
                    "timeSignatureNumerator": 4,
                    "timeSignatureDenominator": 4,
                    "gridDivision": "quarter",
                    "isSnapEnabled": true,
                    "tracks": [],
                    "nextTrackId": 1,
                    "nextClipId": 1
                }
            }"#,
        )
        .expect("project should be created");
        let mut workspaces = Workspaces::new();

        let snapshot = workspaces
            .open(project_file)
            .expect("empty timeline should be normalized");

        assert_eq!(snapshot.timeline.tracks.len(), 1);
        assert_eq!(snapshot.timeline.tracks[0].id, "track-1");
        assert_eq!(snapshot.timeline.tracks[0].name, "Track 1");
    }

    #[test]
    fn saves_and_reopens_timeline_edits() {
        let parent = tempdir().expect("project parent should be created");
        let root = parent.path().join("Arrangement");
        let mut workspaces = Workspaces::new();
        workspaces
            .set_timeline_settings(128.0, 3, 4, GridDivision::Eighth, true)
            .expect("settings should update");
        workspaces
            .save_timeline_track(None, "Drums", false, false)
            .expect("track should save");
        workspaces
            .save_timeline_clip(None, "track-31", "Pattern", 960, 1_920, 0)
            .expect("clip should save");
        let saved = workspaces.save_as(&root).expect("project should save");

        let mut reopened = Workspaces::new();
        let snapshot = reopened
            .open(saved.project_file_path.expect("project should have a path"))
            .expect("project should reopen");

        assert!((snapshot.timeline.bpm - 128.0).abs() < f64::EPSILON);
        assert_eq!(snapshot.timeline.time_signature_numerator, 3);
        assert_eq!(snapshot.timeline.grid_division, GridDivision::Eighth);
        assert_eq!(snapshot.timeline.tracks[30].name, "Drums");
        assert_eq!(snapshot.timeline.tracks[30].clips[0].start_tick, 960);
        assert_eq!(snapshot.timeline.tracks[30].clips[0].duration_ticks, 1_920);
        assert!(!snapshot.is_dirty);
    }

    #[test]
    fn stages_audio_in_memory_and_copies_it_on_first_save() {
        let source_directory = tempdir().expect("source directory should be created");
        let source = source_directory.path().join("Kick.wav");
        fs::write(&source, b"audio").expect("audio source should be created");
        let project_parent = tempdir().expect("project parent should be created");
        let project_root = project_parent.path().join("Drop Test");
        let mut workspaces = Workspaces::new();

        let staged = workspaces
            .import_audio_files([source], "")
            .expect("audio should be staged");
        assert_eq!(staged.files, ["Kick.wav"]);
        assert!(staged.is_dirty);

        let saved = workspaces
            .save_as(&project_root)
            .expect("project should save");
        assert_eq!(saved.files, ["Kick.wav"]);
        assert!(!saved.is_dirty);
        assert_eq!(
            fs::read(project_root.join("Kick.wav")).expect("import should be readable"),
            b"audio"
        );
    }

    #[test]
    fn gives_imports_with_duplicate_names_unique_targets() {
        let sources = tempdir().expect("source directory should be created");
        let first_directory = sources.path().join("one");
        let second_directory = sources.path().join("two");
        fs::create_dir_all(&first_directory).expect("first directory should be created");
        fs::create_dir_all(&second_directory).expect("second directory should be created");
        let first = first_directory.join("Kick.wav");
        let second = second_directory.join("Kick.wav");
        fs::write(&first, b"one").expect("first source should be created");
        fs::write(&second, b"two").expect("second source should be created");
        let mut workspaces = Workspaces::new();

        let snapshot = workspaces
            .import_audio_files([first, second], "")
            .expect("audio should be staged");

        assert_eq!(snapshot.files.len(), 2);
        assert!(snapshot.files.iter().any(|path| path == "Kick.wav"));
        assert!(snapshot.files.iter().any(|path| path == "Kick 2.wav"));
    }

    #[test]
    fn imports_audio_into_the_selected_workspace_directory() {
        let source_directory = tempdir().expect("source directory should be created");
        let source = source_directory.path().join("Snare.wav");
        fs::write(&source, b"audio").expect("audio source should be created");
        let project_parent = tempdir().expect("project parent should be created");
        let project_root = project_parent.path().join("Folder Target");
        let mut workspaces = Workspaces::new();
        workspaces
            .save_as(&project_root)
            .expect("project should save");
        fs::create_dir(project_root.join("Drums")).expect("target directory should be created");

        let snapshot = workspaces
            .import_audio_files([source], "Drums")
            .expect("audio should import");

        assert_eq!(snapshot.files, ["Drums/", "Drums/Snare.wav"]);
        assert_eq!(
            fs::read(project_root.join("Drums/Snare.wav"))
                .expect("imported audio should be readable"),
            b"audio"
        );
    }

    #[test]
    fn rejects_import_targets_outside_the_workspace() {
        let source_directory = tempdir().expect("source directory should be created");
        let source = source_directory.path().join("Snare.wav");
        fs::write(&source, b"audio").expect("audio source should be created");
        let mut workspaces = Workspaces::new();

        let error = workspaces
            .import_audio_files([source], "../outside")
            .expect_err("target should be rejected");

        assert_eq!(
            error.to_string(),
            "audio import target must be a relative project directory"
        );
    }

    #[test]
    fn manages_virtual_directories_and_pending_import_paths() {
        let source_directory = tempdir().expect("source directory should be created");
        let source = source_directory.path().join("Hat.wav");
        fs::write(&source, b"audio").expect("audio source should be created");
        let mut workspaces = Workspaces::new();

        workspaces
            .create_directory("Drums")
            .expect("directory should be created");
        workspaces
            .import_audio_files([source], "Drums")
            .expect("audio should be staged");
        let moved = workspaces
            .move_entry("Drums", "Percussion")
            .expect("directory should move");
        assert_eq!(moved.files, ["Percussion/", "Percussion/Hat.wav"]);

        let deleted = workspaces
            .delete_entry("Percussion/Hat.wav")
            .expect("audio should be removed");
        assert_eq!(deleted.files, ["Percussion/"]);
    }

    #[test]
    fn manages_saved_workspace_entries() {
        let parent = tempdir().expect("project parent should be created");
        let root = parent.path().join("File Operations");
        let mut workspaces = Workspaces::new();
        workspaces.save_as(&root).expect("project should save");

        workspaces
            .create_directory("Samples")
            .expect("directory should be created");
        fs::write(root.join("Samples/Kick.wav"), b"audio").expect("file should be created");
        let moved = workspaces
            .move_entry("Samples/Kick.wav", "Kick.wav")
            .expect("file should move");
        assert_eq!(moved.files, ["Kick.wav", "Samples/"]);

        let deleted = workspaces
            .delete_entry("Samples")
            .expect("directory should be deleted");
        assert_eq!(deleted.files, ["Kick.wav"]);
    }
}
