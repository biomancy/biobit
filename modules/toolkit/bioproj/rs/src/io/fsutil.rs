//! Filesystem operations for project documents, independent of the wire format.

use crate::{Error, Result, Root};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

struct StagedFile {
    // Rust drops fields in declaration order: close the file before cleanup.
    file: File,
    path: StagedPath,
}

struct StagedPath {
    path: PathBuf,
    published: bool,
}

impl StagedFile {
    fn create(directory: &Path) -> Result<Self> {
        static NEXT_FILE: AtomicU64 = AtomicU64::new(0);
        for _ in 0..16 {
            let nonce = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
            let path = directory.join(format!(".bioproj-{}-{nonce}.tmp", std::process::id()));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => {
                    return Ok(Self {
                        file,
                        path: StagedPath {
                            path,
                            published: false,
                        },
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(Error::io(
                        "create temporary project document",
                        Some(&path),
                        error,
                    ));
                }
            }
        }
        Err(Error::io(
            "create temporary project document",
            Some(directory),
            std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "all 16 temporary file names are occupied",
            ),
        ))
    }

    fn publish(self, destination: &Path) -> Result<()> {
        let Self { file, path } = self;
        let synced = file.sync_all();
        drop(file);
        synced
            .map_err(|error| Error::io("synchronize project document", Some(destination), error))?;

        path.publish(destination)
    }
}

impl StagedPath {
    fn publish(mut self, destination: &Path) -> Result<()> {
        fs::rename(&self.path, destination)
            .map_err(|error| Error::io("publish project document", Some(destination), error))?;
        self.published = true;
        Ok(())
    }
}

impl Drop for StagedPath {
    fn drop(&mut self) {
        if !self.published {
            let _ = fs::remove_file(&self.path);
        }
    }
}

pub(super) fn manifest_root(path: &Path) -> Result<Root> {
    let absolute = std::path::absolute(path)
        .map_err(|error| Error::io("resolve project document path", Some(path), error))?;
    let directory = absolute
        .parent()
        .ok_or_else(|| Error::invalid("project document", "must name a file"))?;
    Root::new(directory)
}

pub(super) fn replace(path: &Path, contents: &[u8]) -> Result<()> {
    let directory = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut staged = StagedFile::create(directory)?;
    staged
        .file
        .write_all(contents)
        .map_err(|error| Error::io("write project document", Some(path), error))?;
    match fs::metadata(path) {
        Ok(metadata) => staged
            .file
            .set_permissions(metadata.permissions())
            .map_err(|error| {
                Error::io("preserve project document permissions", Some(path), error)
            })?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(Error::io("inspect output document", Some(path), error)),
    }
    staged.publish(path)
}
