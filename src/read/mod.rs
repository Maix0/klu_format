/*
 *  _  ___     _    _                     _     _
 * | |/ / |   | |  | |     /\            | |   (_)
 * | ' /| |   | |  | |    /  \   _ __ ___| |__  ___   _____
 * |  < | |   | |  | |   / /\ \ | '__/ __| '_ \| \ \ / / _ \
 * | . \| |___| |__| |  / ____ \| | | (__| | | | |\ V /  __/
 * |_|\_\______\____/__/_/____\_\_|__\___|_|_|_|_|_\_/_\___|
 * |______|______|______|______|______|______|______|______|
 *              /  __ \|  ____|   /\   |  __ \
 *              | |__) | |__     /  \  | |  | |
 *              |  _  /|  __|   / /\ \ | |  | |
 *              | | \ \| |____ / ____ \| |__| |
 *              |_|  \_\______/_/    \_\_____/
 */

/*
 * Archive:
 *      0x00 - 0x03: b"KLU "
 *      0x04 - 0x0B: headersize (u64)
 *      0x0C - 0x13: file size  (u64)
 *      0x13 - 0x13 + headersize: File Registrations;
 *      0x14 + headersize - EOF : File Data
 *  File Registrations:
 *                                  0b_______*
 *      0x0: 7b => Filename length; 1b: dir flag (0=dir;1=file)
 *      0x1 - 0x08 => File Size
 *      0x09 + 0x09 + Filename length => Filename
 *  File:
 *      Is a dir:
 *          0x00 - 0x07: Headersize
 *          0x08 - 0x08 + headersize: Dir Header
 *          0x09 - 0x09 + dir size: files data
 *      Is a file:
 *          0x0 - 0x0 + filesize : raw bytes
 */
mod utils;
use std::io::prelude::*;
use std::path::Path;

/// Result type use for reading an archive
pub type ReadResult<T> = Result<T, ReadError>;

#[derive(Debug)]
/// Error used all reading archive functions
pub enum ReadError {
    /// An IO Error
    IoError(std::io::Error),
    /// The archive isn't valid
    InvalidArchive,
    /// The Out-Dir doesn't exist
    InexistantOut,
}

impl std::convert::From<std::io::Error> for ReadError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                Self::IoError(e) => e.to_string(),
                Self::InvalidArchive => "File isn't a valid archive".to_string(),
                Self::InexistantOut => "Path given to release archive do not exist".to_string(),
            }
        )
    }
}

#[derive(Debug)]
/// Main struct of this modules, This represent an archive, allows you to read from it;
pub struct Archive {
    file: File,
    headersize: u64,
    filesize: u64,
    buffer: std::rc::Rc<std::cell::RefCell<std::io::BufReader<std::fs::File>>>,
}

impl Archive {
    /// ID bytes of archive
    pub const ID: [u8; 4] = *b"KLU\x00";

    /// Read an archive from a path
    pub fn from_path<P: AsRef<Path>>(path: P) -> ReadResult<Self> {
        let mut f = std::io::BufReader::new(std::fs::File::open(path)?);
        let mut buffer = vec![0x00; 4 + 8 + 8];
        f.read(&mut buffer)?;
        if buffer[0..4] != Self::ID {
            return Err(ReadError::InvalidArchive);
        }
        let headersize = utils::slice_to_u64(&buffer[4..(4 + 8)]);
        let filesize = utils::slice_to_u64(&buffer[(4 + 8)..(4 + 8 + 8)]);
        buffer = vec![0; headersize as usize];
        f.read(&mut buffer)?;

        Ok(Archive {
            file: File::from_header(&buffer, &mut f, 4 + 8 + 8 + headersize)?,
            buffer: std::rc::Rc::new(std::cell::RefCell::new(f)),
            headersize,
            filesize,
        })
    }

    /// Returns true if a file at given path exists inside the archive
    pub fn path_exist<P: AsRef<Path>>(&mut self, path: P) -> bool {
        match self.get_with_path(path) {
            Some(_) => true,
            None => false,
        }
    }
    fn get_with_path<P: AsRef<Path>>(&self, path: P) -> Option<&File> {
        let path = path
            .as_ref()
            .iter()
            .map(|x| &*x.to_str().unwrap())
            .collect::<Vec<&str>>();
        let mut f = None;
        if path[0] == self.file.filename {
            f = Some(&self.file);
            for name in &path[1..] {
                if let Some(file) = f {
                    let mut flag = false;
                    for child in &file.child {
                        if &child.filename == name {
                            f = Some(child);
                            flag = true;
                        }
                    }
                    if flag == false {
                        f = None;
                    }
                }
            }
        }
        f
    }
}

#[derive(Debug, Clone)]
struct File {
    filename: String,
    filesize: u64,
    is_file: bool,
    child: Vec<Self>,
    relative_offset: u64,
}

impl File {
    fn from_header(
        header: &[u8],
        r_buf: &mut std::io::BufReader<std::fs::File>,
        offset: u64,
    ) -> ReadResult<Self> {
        let (flag, file_size, file_name) = utils::parse_header(header);
        let mut buffer = vec![0_u8; 8];
        let mut childs = Vec::new();
        if !flag {
            r_buf.read(&mut buffer)?;
            let h_size = utils::slice_to_u64(&buffer);
            buffer = vec![0x00; h_size as usize];
            r_buf.read(&mut buffer)?;
            let mut current_offset = offset + 8 + h_size;
            while !buffer.is_empty() {
                let c_header_size = (buffer[0] >> 1) as usize + 8 + 1;
                let c_header = utils::split_in_place(&mut buffer, c_header_size);
                let f = Self::from_header(&c_header, r_buf, current_offset)?;
                current_offset += f.filesize;
                childs.push(f);
            }
        }
        Ok(File {
            filename: file_name,
            filesize: file_size,
            is_file: flag,
            child: childs,
            relative_offset: offset,
        })
    }
}
// Things that help the user, like locating a file with his path...
/// User's function for using an [Archive]
impl Archive {
    /// Extract all archive's content onto a directory
    pub fn release<P: AsRef<Path>>(&mut self, path: P) -> ReadResult<()> {
        if !path.as_ref().exists() {
            return Err(ReadError::InexistantOut);
        }
        let path = path.as_ref().join(self.file.filename.clone());
        self.file
            .write_to_path(&mut self.buffer.try_borrow_mut().unwrap(), path)
    }
    /// Return a `[Vec<String>]` with all files inside the archive
    pub fn paths(self) -> Vec<String> {
        let mut p = Vec::new();
        p.push(format!(
            "{}{}",
            self.file.filename,
            if self.file.is_file { "" } else { "/" }
        ));
        self.file.paths(
            &mut p,
            format!(
                "{}{}",
                self.file.filename,
                if self.file.is_file { "" } else { "/" }
            ),
        );
        p
    }
    /// Extract a single file from the archive
    /// Returns true if the file exists inside the archive, false otherwise
    pub fn extract_file<P: AsRef<Path>>(&mut self, path: P, out: P) -> ReadResult<bool> {
        if let Some(file) = self.get_with_path(path) {
            file.write_to_path(&mut self.buffer.try_borrow_mut().unwrap(), out)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[cfg(feature = "virtual_fs")]
    /// If the path given match a file , returns a [Some(VirtualFile)], else, return [None]
    /// You can have as many [VirtualFile] as you want, even multiples pointing to the same "file",
    /// as they are independend
    pub fn get_virtual<P: AsRef<Path>>(&mut self, path: P) -> Option<VirtualFile> {
        let sizes = match self.get_with_path(path) {
            Some(f) => Some((f.relative_offset as usize, f.filesize as usize)),
            None => None,
        };
        if sizes == None {
            return None;
        }
        Some(VirtualFile::from_sizes(
            sizes.unwrap(),
            std::rc::Rc::clone(&self.buffer),
        ))
    }
}

impl File {
    fn write_to_path<P: AsRef<Path>>(
        &self,
        archive: &mut std::io::BufReader<std::fs::File>,
        output: P,
    ) -> ReadResult<()> {
        if self.is_file {
            let mut file = std::fs::File::create(&output)?;

            let mut remaing = self.filesize;
            let mut buffer = vec![
                0;
                if remaing > 1024 * 1024 {
                    1024 * 1024
                } else {
                    remaing as usize
                }
            ];
            archive.seek(std::io::SeekFrom::Start(self.relative_offset))?;
            while remaing > 0 {
                archive.read(&mut buffer)?;
                file.write(&buffer)?;
                remaing -= buffer.len() as u64;
                buffer = vec![
                    0;
                    if remaing > 1024 * 1024 {
                        1024 * 1024
                    } else {
                        remaing as usize
                    }
                ];
            }
        } else {
            if !output.as_ref().exists() {
                std::fs::create_dir(&output)?;
            }
            for child in &self.child {
                child.write_to_path(archive, output.as_ref().join(child.filename.clone()))?;
            }
        }
        Ok(())
    }
    fn paths(&self, v: &mut Vec<String>, base: String) {
        for file in &self.child {
            if !file.is_file {
                v.push(format!("{}{}/", base, file.filename));
                file.paths(v, format!("{}{}/", base, file.filename));
            } else {
                v.push(format!("{}{}", base, file.filename));
            }
        }
    }
}

#[cfg(feature = "virtual_fs")]
#[derive(Debug, Clone)]
/// Feature: "virtual_fs"
///
/// This represent a file from the archive, it implements [Read] and [Seek] so it can be used with
/// a lot of io-based functions
/// If you need something with [BufRead], just wrap a [std::io::BufReader] around an [VirtualFile]
pub struct VirtualFile {
    buffer: std::rc::Rc<std::cell::RefCell<std::io::BufReader<std::fs::File>>>,
    start_offset: usize,
    end_offset: usize,
    current_offset: usize,
}

#[cfg(feature = "virtual_fs")]
impl VirtualFile {
    /*
    fn from_file(f: &File, b: &'a mut std::io::BufReader<std::fs::File>) -> Self {
        VirtualFile {
            buffer: b,
            start_offset: f.relative_offset as usize,
            end_offset: (f.relative_offset + f.filesize) as usize,
            current_offset: 0,
        }
    }*/
    fn from_sizes(
        s: (usize, usize),
        b: std::rc::Rc<std::cell::RefCell<std::io::BufReader<std::fs::File>>>,
    ) -> Self {
        VirtualFile {
            buffer: b,
            start_offset: s.0,
            end_offset: s.0 + s.1,
            current_offset: 0,
        }
    }
    /// Get the file's data
    pub fn get_slice(&mut self) -> std::io::Result<Box<[u8]>> {
        let mut buf = vec![0; self.end_offset - self.start_offset];
        self.read(&mut buf)?;
        Ok(buf.into_boxed_slice())
    }
}

#[cfg(feature = "virtual_fs")]
impl Read for VirtualFile {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let mut a_buf = self.buffer.try_borrow_mut().unwrap();
        a_buf.seek(std::io::SeekFrom::Start(
            (self.start_offset + self.current_offset) as u64,
        ))?;
        let bytes_left = self.end_offset - self.start_offset - self.current_offset;
        let buffer_len = buffer.len();
        let nbuf_size = if buffer_len < bytes_left {
            buffer_len
        } else {
            bytes_left
        };
        if bytes_left == 0 || buffer.len() == 0 {
            return Ok(0);
        }
        a_buf.read(&mut buffer[0..nbuf_size])?;
        self.current_offset += nbuf_size;
        Ok(nbuf_size)
    }
}
//Os { code: 22, kind: InvalidInput, message: "Invalid argument" }

#[cfg(feature = "virtual_fs")]
impl Seek for VirtualFile {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        use std::io::SeekFrom;
        match pos {
            SeekFrom::Start(n) => {
                self.current_offset = n as usize;
                if self.current_offset + self.start_offset > self.end_offset {
                    self.current_offset = self.end_offset;
                }
            }
            SeekFrom::Current(n) => {
                if (self.current_offset as i64) < n {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Invalid argument",
                    ));
                } else {
                    if (self.current_offset + self.start_offset) as i64 + n > self.end_offset as i64
                    {
                        self.current_offset = self.end_offset;
                    }
                }
            }
            SeekFrom::End(n) => {
                if (self.start_offset as i64) > self.end_offset as i64 - n {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Invalid argument",
                    ));
                }
                if n >= 0 {
                    self.current_offset = self.end_offset;
                } else {
                    self.current_offset = self.end_offset - n as usize;
                }
            }
        }
        let mut buffer = self.buffer.try_borrow_mut().unwrap();
        buffer.seek(std::io::SeekFrom::Start(
            (self.current_offset + self.start_offset) as u64,
        ))?;
        Ok(self.current_offset as u64)
    }
}
