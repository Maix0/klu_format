use crate::types::extract as types;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExtractError {
    #[error("The path does not exist: {0}")]
    DoNotExist(PathBuf),
    #[error("Io Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("The given path exist: {0}")]
    AlreadyExist(PathBuf),
    #[error("The given path isn't a directory: {0}")]
    NotDir(PathBuf),
}

impl types::ExtractOption {
    fn new(path: impl AsRef<Path>) -> Self {
        types::ExtractOption {
            base_path: path.as_ref().to_path_buf(),
            replace: true,
        }
    }
    fn replace(mut self, replace: bool) -> Self {
        self.replace = replace;
        self
    }
}

impl crate::types::read::ArchiveFile {
    #[inline]
    pub fn extract<P: AsRef<Path>>(&self, base_path: P) -> Result<(), ExtractError> {
        self.extract_with_options::<P>(types::ExtractOption::new(base_path))
    }
    fn extract_with_options<P: AsRef<Path>>(
        &self,
        options: types::ExtractOption,
    ) -> Result<(), ExtractError> {
        let base_path = options.base_path;
        if !base_path.exists() {
            return Err(ExtractError::DoNotExist(base_path.clone()));
        }
        if base_path.is_file() {
            return Err(ExtractError::NotDir(base_path.clone()));
        }
        let mut buffer = vec![0; 4096];
        for child in &self.archive.files {
            child.extract_to_path(
                base_path.as_path(),
                &mut std::fs::File::open(&self.file_path)?,
                &mut buffer,
            )?;
        }
        Ok(())
    }
}

impl crate::types::read::File {
    fn extract_to_path<P: AsRef<Path>, A: Read + Seek>(
        &self,
        path: P,
        archive_data: &mut A,
        buffer: &mut Vec<u8>,
    ) -> Result<(), ExtractError> {
        let new_path_buf = {
            let mut p = path.as_ref().to_path_buf();
            p.push(&self.header.filename);
            p
        };
        let new_path = new_path_buf.as_path();
        if self.header.is_file {
            archive_data.seek(std::io::SeekFrom::Start(self.absolute_offset))?;
            let mut file = std::fs::File::create(&new_path)?;
            let mut remain = self.header.filesize as usize;
            while remain > 0 {
                let bytes = archive_data.read(buffer)?;
                file.write(&buffer[..(bytes.min(remain))])?;
                remain = remain.checked_sub(bytes).unwrap_or(0);
            }
            Ok(())
        } else {
            std::fs::create_dir(new_path)?;
            for child in &self.childs {
                child.extract_to_path(new_path, archive_data, buffer)?;
            }
            Ok(())
        }
    }
}
