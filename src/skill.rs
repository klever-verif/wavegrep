use std::fs;
use std::path::{Component, Path, PathBuf};

use include_dir::{Dir, include_dir};
use serde::Serialize;

use crate::error::WavepeekError;

static BUNDLE: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/skills/wavepeek");

#[derive(Serialize)]
struct Manifest {
    wavepeek_version: &'static str,
}

pub fn materialize(destination: &Path) -> Result<(), WavepeekError> {
    if destination
        .components()
        .any(|component| component == Component::ParentDir)
    {
        return Err(file_error(format!(
            "skill destination '{}' must not contain '..'",
            destination.display()
        )));
    }
    validate_destination(destination)?;

    let parent = destination
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| {
        file_error(format!(
            "failed to create parent directory '{}': {error}",
            parent.display()
        ))
    })?;

    let staging = create_staging_dir(parent, destination)?;
    let result = write_bundle(&staging).and_then(|()| install_bundle(&staging, destination));
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

fn validate_destination(destination: &Path) -> Result<(), WavepeekError> {
    let metadata = match fs::symlink_metadata(destination) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(file_error(format!(
                "failed to inspect skill destination '{}': {error}",
                destination.display()
            )));
        }
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(file_error(format!(
            "skill destination '{}' is not a directory",
            destination.display()
        )));
    }
    if fs::read_dir(destination)
        .map_err(|error| {
            file_error(format!(
                "failed to inspect skill destination '{}': {error}",
                destination.display()
            ))
        })?
        .next()
        .is_some()
    {
        return Err(file_error(format!(
            "skill destination '{}' is not empty",
            destination.display()
        )));
    }
    Ok(())
}

fn create_staging_dir(parent: &Path, destination: &Path) -> Result<PathBuf, WavepeekError> {
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("wavepeek-skill");
    for attempt in 0..100 {
        let staging = parent.join(format!(
            ".{name}.wavepeek-tmp-{}-{attempt}",
            std::process::id()
        ));
        match fs::create_dir(&staging) {
            Ok(()) => return Ok(staging),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(file_error(format!(
                    "failed to create temporary skill directory '{}': {error}",
                    staging.display()
                )));
            }
        }
    }
    Err(file_error(format!(
        "failed to allocate a temporary skill directory beside '{}'",
        destination.display()
    )))
}

fn write_bundle(staging: &Path) -> Result<(), WavepeekError> {
    write_dir(&BUNDLE, staging)?;
    let mut manifest = serde_json::to_vec_pretty(&Manifest {
        wavepeek_version: env!("CARGO_PKG_VERSION"),
    })
    .map_err(|error| file_error(format!("failed to serialize skill manifest: {error}")))?;
    manifest.push(b'\n');
    fs::write(staging.join("manifest.json"), manifest)
        .map_err(|error| file_error(format!("failed to write skill manifest: {error}")))
}

fn write_dir(dir: &Dir<'_>, root: &Path) -> Result<(), WavepeekError> {
    for child in dir.dirs() {
        fs::create_dir_all(root.join(child.path())).map_err(|error| {
            file_error(format!(
                "failed to create bundled directory '{}': {error}",
                child.path().display()
            ))
        })?;
        write_dir(child, root)?;
    }
    for file in dir.files() {
        if file.path().file_name().and_then(|name| name.to_str()) == Some(".gitkeep") {
            continue;
        }
        let path = root.join(file.path());
        fs::write(&path, file.contents()).map_err(|error| {
            file_error(format!(
                "failed to write bundled file '{}': {error}",
                file.path().display()
            ))
        })?;
    }
    Ok(())
}

fn install_bundle(staging: &Path, destination: &Path) -> Result<(), WavepeekError> {
    let existed = destination.exists();
    if existed {
        validate_destination(destination)?;
    }
    let Err(initial_error) = fs::rename(staging, destination) else {
        return Ok(());
    };
    if existed {
        validate_destination(destination)?;
        fs::remove_dir(destination).map_err(|error| {
            file_error(format!(
                "failed to prepare empty skill destination '{}': {error}",
                destination.display()
            ))
        })?;
        if let Err(error) = fs::rename(staging, destination) {
            if !destination.exists() {
                fs::create_dir(destination).map_err(|restore_error| {
                    file_error(format!(
                        "failed to install skill at '{}' ({error}) and restore the empty destination: {restore_error}",
                        destination.display()
                    ))
                })?;
            }
            return Err(file_error(format!(
                "failed to install skill at '{}': {error}",
                destination.display()
            )));
        }
        return Ok(());
    }
    Err(file_error(format!(
        "failed to install skill at '{}': {initial_error}",
        destination.display()
    )))
}

fn file_error(message: String) -> WavepeekError {
    WavepeekError::File(message)
}
