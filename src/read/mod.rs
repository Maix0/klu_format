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

pub fn test() {}

#[derive(Debug)]
pub struct Archive {
    file: File,
    headersize: u64,
    filesize: u64,
    buffer: std::io::BufReader<std::fs::File>,
}

impl Archive {
    pub const ID: [u8; 4] = *b"KLU\x00";

    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let mut f = std::io::BufReader::new(
            std::fs::File::open(path).expect("Error while opening archive"),
        );
        let mut buffer = vec![0x00; 4 + 8 + 8];
        f.read(&mut buffer)
            .expect("Error while reading archive defining bytes");
        if buffer[0..4] != Self::ID {
            panic!("File isn't a valid archive");
        }
        let headersize = utils::slice_to_u64(&buffer[4..(4 + 8)]);
        let filesize = utils::slice_to_u64(&buffer[(4 + 8)..(4 + 8 + 8)]);
        buffer = vec![0; headersize as usize];
        f.read(&mut buffer)
            .expect("Error while reading main file header");

        Archive {
            file: File::from_header(&buffer, &mut f, 4 + 8 + 8 + headersize),
            buffer: f,
            headersize,
            filesize,
        }
    }
}

#[derive(Debug)]
pub struct File {
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
    ) -> Self {
        let (flag, file_size, file_name) = utils::parse_header(header);
        let mut buffer = vec![0_u8; 8];
        let mut childs = Vec::new();
        if !flag {
            r_buf
                .read(&mut buffer)
                .expect("Error while reading file's headersize");
            let h_size = utils::slice_to_u64(&buffer);
            buffer = vec![0x00; h_size as usize];
            r_buf
                .read(&mut buffer)
                .expect("Error while reading file's headers");
            let mut current_offset = offset + 8 + h_size;
            while !buffer.is_empty() {
                let c_header_size = (buffer[0] >> 1) as usize + 8 + 1;
                let c_header = utils::split_in_place(&mut buffer, c_header_size);
                let f = Self::from_header(&c_header, r_buf, current_offset);
                current_offset += f.filesize;
                childs.push(f);
            }
        }
        File {
            filename: file_name,
            filesize: file_size,
            is_file: flag,
            child: childs,
            relative_offset: offset,
        }
    }
}
// Things that help the user, like locating a file with his path...
/// User's function for using an [Archive]
impl Archive {
    /*pub fn get_file_at<P: AsRef<Path>>(path: P) -> Option<File> {
        for
    }*/
    pub fn release<P: AsRef<Path>>(&mut self, path: P) {
        if !path.as_ref().exists() {
            panic!("Given path does not exist")
        }
        let path = path.as_ref().join(self.file.filename.clone());
        if path.exists() {
            panic!("Path where archive will be released already exist");
        }
        self.file.write_to_path(&mut self.buffer, path);
    }
}

impl File {
    fn write_to_path<P: AsRef<Path>>(
        &self,
        archive: &mut std::io::BufReader<std::fs::File>,
        output: P,
    ) {
        if self.is_file {
            let mut file = std::fs::File::create(&output).expect("Error while creating file");
            let mut remaing = self.filesize;
            let mut buffer = vec![
                0;
                if remaing > 1024 * 1024 {
                    1024 * 1024
                } else {
                    remaing as usize
                }
            ];
            archive
                .seek(std::io::SeekFrom::Start(self.relative_offset))
                .expect("Error while setting read cursor at start of the file");
            while remaing > 0 {
                archive
                    .read(&mut buffer)
                    .expect("Error while reading out from the archive");
                file.write(&buffer).expect("Error while writing to file");
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
            std::fs::create_dir(&output).expect("Error while creating a dir");
            for child in &self.child {
                child.write_to_path(archive, output.as_ref().join(child.filename.clone()));
            }
        }
    }
}
