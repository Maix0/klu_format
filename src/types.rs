pub mod read {
    #![cfg_attr(debug_assertions, allow(dead_code))]
    use std::path::{Path, PathBuf};
    pub(crate) const MAGIC: [u8; 4] = *b"KLU\0";
    #[derive(Debug, Eq, PartialEq, Clone)]
    pub(crate) struct Archive {
        pub(crate) filesize: u64,
        pub(crate) headersize: u64,
        pub(crate) files: Vec<File>,
    }
    #[derive(Debug, Eq, PartialEq, Clone)]
    pub(crate) struct File {
        pub(crate) header: Header,
        pub(crate) absolute_offset: u64,
        pub(crate) childs: Vec<File>,
    }

    #[derive(Debug, Eq, PartialEq, Clone)]
    pub(crate) struct Header {
        pub(crate) filename_length: u8, // 5 bits;
        pub(crate) unused: u8,          // 2 bits
        pub(crate) is_file: bool,       // true = File, false = Folder;
        pub(crate) filesize: u64,       // 8 bytes
        pub(crate) filename: String,    // max 31 bytes
    }
    #[derive(Debug, Eq, Clone, PartialEq)]
    pub struct ArchiveFile {
        pub(crate) file_path: PathBuf,
        pub(crate) archive: super::read::Archive,
    }
}

pub(crate) mod extract {
    use std::path::PathBuf;
    #[derive(Debug, Clone)]
    pub struct ExtractOption {
        pub(crate) base_path: PathBuf, // Base path of the extraction
        pub(crate) replace: bool,      // Replace files, file -> dir and such
    }
}

pub(crate) mod write {
    use std::path::PathBuf;
    #[derive(Debug, Eq, Clone, PartialEq)]
    pub(crate) struct Archive {
        pub(crate) base_path: PathBuf,
        pub(crate) childs: Vec<File>,
    }

    #[derive(Debug, Eq, Clone, PartialEq)]
    pub(crate) struct File {
        pub(crate) path: PathBuf,
        pub(crate) header: Header,
        pub(crate) childs: Vec<File>,
    }

    #[derive(Debug, Eq, PartialEq, Clone)]
    pub(crate) struct Header {
        pub(crate) filename_length: u8, // 5 bits;
        pub(crate) unused: u8,          // 2 bits
        pub(crate) is_file: bool,       // true = File, false = Folder;
        pub(crate) filesize: u64,       // 8 bytes
        pub(crate) filename: String,    // max 31 bytes
    }

    impl File {
        pub(crate) fn file_size(&self) -> u64 {
            if self.header.is_file {
                self.header.filesize
            } else {
                let mut size = 8;
                for child in &self.childs {
                    size += child.header_size() + child.file_size();
                }
                size
            }
        }
        pub(crate) fn header_size(&self) -> u64 {
            if self.header.is_file {
                0
            } else {
                let mut size = 0;
                for child in &self.childs {
                    size += 1 + 8 + child.header.filename_length as u64;
                }
                size
            }
        }

        #[inline]
        pub(crate) fn update_size(mut self) -> Self {
            self.header.filesize = self.file_size();
            self
        }
    }
}
