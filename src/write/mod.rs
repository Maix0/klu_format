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
use std::path::PathBuf;
use std::io::prelude::*;
mod utils;
#[derive(Debug)]
pub struct Archive {
    headersize: u64,
    filesize: u64,
    file: File,
}

impl Archive {
    /// The 4 bytes at the start of any archive
    pub const ID: [u8; 4] = *b"KLU\x00";
    /// Create an archive from the path
    pub fn from_path(path: PathBuf) -> Self {
        let file = File::from_path(path);
        let filesize =  Self::ID.len() as u64 + 
                        8 /* headersize */ + 
                        8 /* filesize */ + 
                        file.header_len() as u64 + file.filesize;
        Archive {
            headersize: file.header_len() as u64,
            filesize: filesize,
            file: file,
        }
    }
    ///Write archive to file at given path. Will create a new file or truncate it if allready
    ///existing
    pub fn write_to_path(&mut self, path:PathBuf) {
        let out_file = std::fs::File::create(path)
            .expect("Couldn't create/truncate archive file");
        let mut buffer = std::io::BufWriter::new(out_file);
        buffer.write(&Self::ID).expect("Error while writing ID bytes to buffer");
        buffer.write(&utils::u64_to_slice(self.headersize)).
            expect("Error while writing header_size to buffer");
        buffer.write(&utils::u64_to_slice(self.filesize)).
            expect("Error while writing file_size to buffer");
        buffer.write(&self.file.header()).expect("Error while writing main file to buffer");
        self.file.write_to_buf(&mut buffer);
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
    pub fn write_to_buf<W:Write>(&self, buffer: &mut std::io::BufWriter<W>) {
        if self.is_file {
            let mut reader = std::io::BufReader::new(std::fs::File::open(&self.path)
                .expect("Can't open file for writing archive"));
            loop {
                let length;
                {
                    let bytes = reader.fill_buf().unwrap();
                    length = bytes.len();
                    if bytes.is_empty() {
                        break;
                    }
                    buffer.write(bytes).expect("Error while writing to buffer");
                }
                reader.consume(length);   
            }
            buffer.flush().expect("Error while flushing buffer");
        } else {
            let mut headersize = 0;
            for c in &self.childs {
                headersize += c.header_len() as u64;
            }
            buffer.write(&utils::u64_to_slice(headersize))
                .expect("Error while writing to buffer");
            for c in &self.childs {   
                buffer.write(&c.header())
                    .expect("Error while writing to buffer");
            }
            for c in &self.childs {
                c.write_to_buf(buffer);
            }
        }
    }

    /// Create a [File] from a [PathBuf], will populate childs if needed
    pub fn from_path(path: PathBuf) -> Self {
        let path = path
            .canonicalize()
            .expect("Error while canonicalizing the path");
        let md = path
            .metadata()
            .expect("Error while getting path's metadata");
        let mut filesize = if md.is_file() { md.len() } else { 8 };
        if let Some(fname) = path.file_name() {
            if None == fname.to_str() {
                panic!(format!("Filename `{}` isn't valid UTF-8", path.display()));
            }
            if fname.len() > 127 {
                panic!(format!("Filename `{:?}` is longer than 127 chars", fname));
            }
        } else {
            panic!(format!("Filename `{}` doesn't exist", path.display()));
        }
        let mut childs = Vec::new();
        if md.is_dir() {
            for dir in path.read_dir().unwrap() {
                if let Ok(child) = dir {
                    let child_file = Self::from_path(child.path());
                    filesize += child_file.header_len() as u64 + child_file.filesize;
                    childs.push(child_file);
                }
            }
        }
        File {
            filesize: filesize,
            is_file: md.is_file(),
            filename: path.file_name().unwrap().to_str().unwrap().to_owned(),
            path: path,
            childs: childs,
        }
    }
}
