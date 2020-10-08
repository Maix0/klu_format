pub mod read {
    #![cfg_attr(debug_assertions, allow(dead_code))]
    pub const MAGIC: [u8; 4] = *b"KLU\0";
    #[derive(Debug, Eq, PartialEq, Clone)]
    pub struct Archive {
        pub filesize: u64,
        pub headersize: u64,
        pub files: Vec<File>,
    }
    #[derive(Debug, Eq, PartialEq, Clone)]
    pub struct File {
        pub header: Header,
        pub absolute_offset: u64,
        pub childs: Vec<File>,
    }

    #[derive(Debug, Eq, PartialEq, Clone)]
    pub struct Header {
        pub filename_length: u8, // 5 bits;
        pub unused: u8,          // 2 bits
        pub is_file: bool,       // true = File, false = Folder;
        pub filesize: u64,       // 8 bytes
        pub filename: String,    // max 31 bytes
    }
}

pub mod write {
    use std::path::{Path, PathBuf};
    #[derive(Debug, Eq, Clone, PartialEq)]
    pub struct Archive {
        pub base_path: PathBuf,
        pub childs: Vec<File>,
    }

    #[derive(Debug, Eq, Clone, PartialEq)]
    pub struct File {
        pub path: PathBuf,
        pub header: Header,
        pub childs: Vec<File>,
    }

    #[derive(Debug, Eq, PartialEq, Clone)]
    pub struct Header {
        pub filename_length: u8, // 5 bits;
        pub unused: u8,          // 2 bits
        pub is_file: bool,       // true = File, false = Folder;
        pub filesize: u64,       // 8 bytes
        pub filename: String,    // max 31 bytes
    }

    impl File {
        pub fn file_size(&self) -> u64 {
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
        pub fn header_size(&self) -> u64 {
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
        pub fn update_size(mut self) -> Self {
            self.header.filesize = self.file_size();
            self
        }
    }
}
