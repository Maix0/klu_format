use crate::types;
use nom::{bits::complete as par_bit, bytes::complete as par, IResult};

pub type NomReadErr<'a, T> = IResult<&'a [u8], T, (&'a [u8], ReadError)>;
#[derive(Debug, Error, Eq, PartialEq)]
pub enum ReadError {
    #[error("Unknown Error\n Near: 0x{0:X?}")]
    Unknown(u64),
    #[error("Expected {:?}, found {:?}\n Near: 0x{1:X?}", types::read::MAGIC, u32::to_be_bytes(*.0))]
    WrongMagic(u32, u64),
    #[error("Missing information bytes in file header\n Near: 0x{0:X?}")]
    MissingHeaderInfoByte(u64),
    #[error("Missing header file size\n Near: 0x{0:X?}")]
    MissingHeaderFileSize(u64),
    #[error("File name isn't UTF-8\n Near: 0x{0:X?}")]
    IllegalFilename(u64),
    #[error("Missing Archivesize or Headersize\n Near: 0x{0:X?}")]
    MissingArchiveInfo(u64),
    #[error("Incomplete Header\n Near: 0x{0:X?}")]
    IncompleteHeader(u64),
    #[error("Filejump data Erro\n Near: 0x{0:X?}r")]
    FileJump(u64),
    #[error("Missing FileInfo\n Near: 0x{0:X?}")]
    MissingFileInfo(u64),
}
pub fn parse_magic(b: &[u8], file_offset: u64) -> NomReadErr<'_, &[u8]> {
    par::tag::<&[u8], &[u8], (&[u8], nom::error::ErrorKind)>(b"KLU\0")(b).map_err(|e| {
        e.map(|inner| {
            use std::convert::TryInto;
            let o: [u8; 4] = inner.0.try_into().unwrap_or([0xFF; 4]);
            (
                inner.0,
                ReadError::WrongMagic(u32::from_be_bytes(o), file_offset),
            )
        })
    })
}
#[inline]
fn parse_bit<E>(e: nom::Err<E>) -> nom::Err<E> {
    if let nom::Err::Incomplete(_) = e {
        nom::Err::Incomplete(nom::Needed::Size(1))
    } else {
        e
    }
}

pub fn parse_header(b: &[u8], file_offset: u64) -> NomReadErr<'_, types::read::Header> {
    let ((input, pos), filename_length) =
        par_bit::take::<&[u8], u8, u32, ()>(5)((b, 0)).map_err(|e: nom::Err<_>| {
            parse_bit(e.map(|_| (b, ReadError::MissingHeaderInfoByte(file_offset))))
        })?;
    let ((input, pos), unused) =
        par_bit::take::<&[u8], u8, u32, ()>(2)((input, pos)).map_err(|e: nom::Err<_>| {
            parse_bit(e.map(|_| (b, ReadError::MissingHeaderInfoByte(file_offset))))
        })?;
    let ((input, _), is_file) =
        par_bit::take::<&[u8], u8, u32, ()>(1)((input, pos)).map_err(|e: nom::Err<_>| {
            parse_bit(e.map(|_| (b, ReadError::MissingHeaderInfoByte(file_offset))))
        })?;
    let (input, filesize) = nom::number::streaming::be_u64::<(_, nom::error::ErrorKind)>(input)
        .map_err(|e| e.map(|e| (e.0, ReadError::MissingHeaderFileSize(file_offset))))?;
    let (input, str_bytes) =
        par::take::<u8, &[u8], (&[u8], nom::error::ErrorKind)>(filename_length)(input)
            .map_err(|e| e.map(|e| (e.0, ReadError::Unknown(file_offset))))?;
    let filename_str = std::str::from_utf8(str_bytes)
        .map_err(|_| nom::Err::Failure((input, ReadError::IllegalFilename(file_offset))))?
        .to_string();

    Ok((
        input,
        types::read::Header {
            filename_length,
            unused,
            is_file: is_file == 1,
            filesize,
            filename: filename_str,
        },
    ))
}

pub fn parse_archive(b: &[u8]) -> NomReadErr<'_, types::read::Archive> {
    let (input, _magic) = parse_magic(b, 0)?;
    let (input, headersize) = nom::number::streaming::be_u64::<(_, nom::error::ErrorKind)>(input)
        .map_err(|e| e.map(|e| (e.0, ReadError::MissingArchiveInfo(4))))?;
    let (input, filesize) = nom::number::streaming::be_u64::<(_, nom::error::ErrorKind)>(input)
        .map_err(|e| e.map(|e| (e.0, ReadError::MissingArchiveInfo(12))))?;
    let (input, header_data) =
        par::take::<u64, &[u8], (&[u8], nom::error::ErrorKind)>(headersize)(input)
            .map_err(|e| e.map(|e1: (&[u8], _)| (e1.0, ReadError::IncompleteHeader(20))))?;
    let (_, headers) = parse_headerlist(header_data)?;
    let (input, files) = parse_filelist(input, headers, 4 + 8 + 8 + headersize)?; //Vec::new();
    Ok((
        input,
        types::read::Archive {
            headersize,
            filesize,
            files,
        },
    ))
}

pub fn parse_headerlist(b: &[u8], file_offset: u64) -> NomReadErr<'_, Vec<types::read::Header>> {
    let mut header_list: Vec<types::read::Header> = Vec::new();
    let mut current_input = b;
    while !current_input.is_empty() {
        let (new_input, header) = parse_header(current_input)?;
        current_input = new_input;
        header_list.push(header);
    }
    Ok((current_input, header_list))
}

pub fn parse_file(
    b: &[u8],
    header: types::read::Header,
    offset: u64,
    file_offset: u64,
) -> NomReadErr<'_, types::read::File> {
    if header.is_file {
        let (input, headersize) = nom::number::streaming::be_u64::<(_, nom::error::ErrorKind)>(b)
            .map_err(|e| e.map(|e| (e.0, ReadError::MissingFileInfo)))?;
        let (input, header_data) =
            par::take::<u64, &[u8], (&[u8], nom::error::ErrorKind)>(headersize)(input)
                .map_err(|e| e.map(|e1: (&[u8], _)| (e1.0, ReadError::IncompleteHeader)))?;
        let (_, headers) = parse_headerlist(header_data)?;
        let (input, files) = parse_filelist(input, headers, 8 + headersize + offset)?;
        Ok((
            input,
            types::read::File {
                header,
                absolute_offset: offset,
                childs: files,
            },
        ))
    } else {
        // TODO: Jump here
        let (rest, _jump) =
            par::take::<u64, &[u8], (&[u8], nom::error::ErrorKind)>(header.filesize)(b)
                .map_err(|e| e.map(|e1: (&[u8], _)| (e1.0, ReadError::FileJump)))?;
        Ok((
            rest,
            types::read::File {
                header,
                absolute_offset: offset,
                childs: Vec::new(),
            },
        ))
    }
}

pub fn parse_filelist(
    b: &[u8],
    headers: Vec<types::read::Header>,
    mut offset: u64,
    file_offset: u64,
) -> NomReadErr<'_, Vec<types::read::File>> {
    let mut files = Vec::new();
    let mut input = b;
    for header in headers {
        let fsize = header.filesize;
        let (i, file) = parse_file(input, header, offset, file_offset)?;
        input = i;
        files.push(file);
        offset += fsize;
        file_offset += 1 + 8 + header.filename_length as u64;
    }
    Ok((input, files))
}
