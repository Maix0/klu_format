/*
 *  _  ___     _    _                     _     _
 * | |/ / |   | |  | |     /\            | |   (_)
 * | ' /| |   | |  | |    /  \   _ __ ___| |__  ___   _____
 * |  < | |   | |  | |   / /\ \ | '__/ __| '_ \| \ \ / / _ \
 * | . \| |___| |__| |  / ____ \| | | (__| | | | |\ V /  __/
 * |_|\_\______\____/__/_/____\_\_|__\___|_|_|_|_|_\_/_\___|
 * |______|______|______|______|______|______|______|______|
 *         \ \        / /  __ \|_   _|__   __|  ____|
 *          \ \  /\  / /| |__) | | |    | |  | |__
 *           \ \/  \/ / |  _  /  | |    | |  |  __|
 *            \  /\  /  | | \ \ _| |_   | |  | |____
 *             \/  \/   |_|  \_\_____|  |_|  |______|
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
use std::path::{PathBuf,Path};
use std::io::prelude::*;
mod utils;
#[derive(Debug)]
pub struct Archive {
    headersize: u64,
    filesize: u64,
    file: File,
}

pub type WriteResult<T> = Result<T,WriteError>;

#[derive(Debug)]
pub enum WriteError {
    IoError(std::io::Error),
    InvalidInput(Filename)
}

#[derive(Debug)]
pub enum Filename {
    NotUTF8(String),
    TooLong(String),
    Inexistant(String)
}

impl std::convert::From<std::io::Error> for WriteError {
    fn from(err: std::io::Error) -> Self{
        Self::IoError(err)
    }
}

impl Archive {
    /// The 4 bytes at the start of any archive
    const ID: [u8; 4] = *b"KLU\x00";
    /// Create an archive from the path
    pub fn from_path<P:AsRef<Path>>(path: P) -> WriteResult<Self> {
        let file = File::from_path(path)?;
        let filesize =  Self::ID.len() as u64 + 
                        8 /* headersize */ + 
                        8 /* filesize */ + 
                        file.header_len() as u64 + file.filesize;
        Ok(Archive {
            headersize: file.header_len() as u64,
            filesize: filesize,
            file: file,
        })
    }
    ///Write archive to file at given path. Will create a new file or truncate it if allready
    ///existing
    pub fn write_to_path<P:AsRef<Path>>(&self, path : P) -> WriteResult<()> {
        let out_file = std::fs::File::create(path)?;
        let mut buffer = std::io::BufWriter::new(out_file);
        buffer.write(&Self::ID)?;
        buffer.write(&utils::u64_to_slice(self.headersize))?;
        buffer.write(&utils::u64_to_slice(self.filesize))?;
        buffer.write(&self.file.header())?;
        self.file.write_to_buf(&mut buffer)?;
        Ok(())
    } 
}

#[derive(Debug)]
/// Represent a file on the archive
pub struct File {
    filesize: u64,
    is_file: bool,
    filename: String,
    path: PathBuf,
    childs: Vec<File>,
}

impl File {
    ///Get the file header
    pub fn header(&self) -> Box<[u8]> {
        let mut header = vec![0x00_u8; 1];
        header[0x00] = (self.filename.len() << 1) as u8 | self.is_file as u8;
        header = [&*header, &*utils::u64_to_slice(self.filesize)].concat();
        header = [&*header, self.filename.as_bytes()].concat();
        header.into_boxed_slice()
    }
    /// Return the file's header length
    pub fn header_len(&self) -> usize {
        return 1 /*filename length + dir bit*/ + 8 /*filesize (u64)*/ + self.filename.len();
    }
    /// Write file to given buffer, needs to be a mutable reference because it 
    /// will be given to file's children an so on;
    pub fn write_to_buf<W:Write>(&self, buffer: &mut std::io::BufWriter<W>) -> WriteResult<()>{
        if self.is_file {
            let mut reader = std::io::BufReader::new(std::fs::File::open(&self.path)?);
            std::io::copy(&mut reader,buffer)?;
            buffer.flush()?;
        } else {
            let mut headersize = 0;
            for c in &self.childs {
                headersize += c.header_len() as u64;
            }
            buffer.write(&utils::u64_to_slice(headersize))?;
            for c in &self.childs {   
                buffer.write(&c.header())?;
            }
            for c in &self.childs {
                c.write_to_buf(buffer)?;
            }
        }
        Ok(())
    }

    /// Create a [File] from a [PathBuf], will populate childs if needed
    pub fn from_path<P:AsRef<Path>>(path: P) -> WriteResult<Self> {
        let path = path.as_ref()
            .canonicalize()?;
        let md = path
            .metadata()?;
        let mut filesize = if md.is_file() { md.len() } else { 8 };
        if let Some(fname) = path.file_name() {
            if None == fname.to_str() {
                return Err(WriteError::InvalidInput(Filename::NotUTF8(
                            format!("Filename `{}` isn't valid UTF-8", path.display()))));
            }
            if fname.len() > 127 {
                return Err(WriteError::InvalidInput(
                        Filename::TooLong(
                            format!("Filename `{:?}` is longer than 127 chars", fname)))); 
            }
        } else {
            return Err(WriteError::InvalidInput(Filename::Inexistant(
                        format!("Filename `{}` doesn't exist", path.display()))));
        }
        let mut childs = Vec::new();
        if md.is_dir() {
            for dir in path.read_dir()? {
                if let Ok(child) = dir {
                    let child_file = Self::from_path(child.path())?;
                    filesize += child_file.header_len() as u64 + child_file.filesize;
                    childs.push(child_file);
                }
            }
        }
        Ok(File {
            filesize: filesize,
            is_file: md.is_file(),
            filename: path.file_name().unwrap().to_str().unwrap().to_owned(),
            path: path,
            childs: childs,
        })
    }
}
