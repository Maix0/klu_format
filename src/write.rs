#![cfg_attr(debug_assertions, allow(dead_code))]

use crate::types::write as types;
use std::path::{Path, PathBuf};
#[derive(Debug, Error)]
pub enum WriteError {
    #[error("Filename `{0:?}` is too long (> 31)")]
    FileNameTooLong(PathBuf),
    #[error("Filename {0:?} isn't UTF-8")]
    FilenameNotUTF8(PathBuf),
    #[error("Unknown Error due to Filename")]
    FilenameError(PathBuf),
    #[error("IoError: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Unknown Error")]
    Unknown,
}
pub type WriteResult<T> = Result<T, WriteError>;

pub fn path_to_archive<P: AsRef<Path>>(path: P) -> WriteResult<types::Archive> {
    let metadata = std::fs::metadata(&path)?;
    if metadata.is_dir() {
        let filename = path
            .as_ref()
            .file_name()
            .ok_or(WriteError::FilenameError(path.as_ref().to_path_buf()))?
            .to_str()
            .ok_or(WriteError::FilenameNotUTF8(path.as_ref().to_path_buf()))?
            .to_string();
        if filename.bytes().len() > 31 {
            return Err(WriteError::FilenameNotUTF8(path.as_ref().to_path_buf()));
        }
        let header = types::Header {
            unused: 0,
            filename_length: filename.bytes().len() as u8,
            filename,
            is_file: false,
            filesize: metadata.len(),
        };
        todo!()
    } else {
        let file = path_to_file(&path)?;
        Ok(types::Archive {
            base_path: path.as_ref().to_path_buf(),
            childs: vec![file],
        })
    }
}

pub fn path_to_file<P: AsRef<Path>>(path: P) -> WriteResult<types::File> {
    let metadata = std::fs::metadata(&path)?;
    if metadata.is_dir() {
        let filename = path
            .as_ref()
            .file_name()
            .ok_or(WriteError::FilenameError(path.as_ref().to_path_buf()))?
            .to_str()
            .ok_or(WriteError::FilenameNotUTF8(path.as_ref().to_path_buf()))?
            .to_string();
        if filename.bytes().len() > 31 {
            return Err(WriteError::FilenameNotUTF8(path.as_ref().to_path_buf()));
        }
        let header = types::Header {
            unused: 0,
            filename_length: filename.bytes().len() as u8,
            filename,
            is_file: false,
            filesize: metadata.len(),
        };
        let mut files = Vec::new();
        for r in std::fs::read_dir(&path.as_ref())?
            .into_iter()
            .map(|r| r.map(|d| path_to_file(d.path())))
        {
            files.push(r??);
        }

        Ok(types::File {
            header,
            path: path.as_ref().to_path_buf(),
            childs: files,
        }
        .update_size())
    } else {
        let filename = path
            .as_ref()
            .file_name()
            .ok_or(WriteError::FilenameError(path.as_ref().to_path_buf()))?
            .to_str()
            .ok_or(WriteError::FilenameNotUTF8(path.as_ref().to_path_buf()))?
            .to_string();
        if filename.bytes().len() > 31 {
            return Err(WriteError::FilenameNotUTF8(path.as_ref().to_path_buf()));
        }
        let header = types::Header {
            unused: 0,
            filename_length: filename.bytes().len() as u8,
            filename,
            is_file: true,
            filesize: metadata.len(),
        };
        Ok(types::File {
            header,
            path: path.as_ref().to_path_buf(),
            childs: Vec::new(),
        }
        .update_size())
    }
}
