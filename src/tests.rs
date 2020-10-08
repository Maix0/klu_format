#[macro_export]
macro_rules! include_testfile {
    ($file:literal) => {
        include_bytes!(concat!(
            concat!(env!("CARGO_MANIFEST_DIR"), "/testsdata/"),
            $file
        )) as &[u8]
    };
}
#[macro_export]
macro_rules! path_testfile {
    ($file:literal) => {
        concat!(concat!(env!("CARGO_MANIFEST_DIR"), "/testsdata/"), $file)
    };
}
mod read {
    use crate::read::{parse_archive, parse_header, parse_magic, NomReadErr, ReadError};
    use crate::types::read as types;
    #[test]
    fn magic_correct() {
        assert_eq!(
            parse_magic(include_testfile!("magic_correct")),
            Ok((&[] as &[u8], &types::MAGIC as &[u8])) as NomReadErr<'_, &[u8]>
        );
    }
    // #[test]
    // fn magic_incomplete() {
    //     assert_eq!(
    //         parse_magic(include_testfile!("magic_incompete")),
    //         Err(nom::Err::Incomplete(nom::Needed::Size(4))) as NomReadErr<'_, &[u8]>
    //     );
    // }
    #[test]
    fn magic_wrong() {
        assert_eq!(
            parse_magic(include_testfile!("magic_wrong")),
            Err(nom::Err::Error((
                &[0xDEu8, 0xAD, 0xBE, 0xEF] as &[u8],
                ReadError::WrongMagic(0xDEADBEEF)
            ))) as NomReadErr<'_, &[u8]>
        );
    }
    #[test]
    fn header_correct() {
        assert_eq!(
            parse_header(include_testfile!("header_correct")),
            Ok((
                &[] as &[u8],
                types::Header {
                    filename_length: 28,
                    unused: 0,
                    is_file: false,
                    filesize: 74,
                    filename: String::from("ceci est un long text de 28 "),
                },
            ),)
        );
    }
    // #[test]
    // fn header_incomplete() {
    //     assert_eq!(
    //         parse_header(include_testfile!("header_incompete")),
    //         Err(nom::Err::Incomplete(nom::Needed::Size(28)))
    //     );
    // }
    #[test]
    fn header_wrong() {
        assert_eq!(
            parse_header(include_testfile!("header_wrong")),
            Err(nom::Err::Failure((
                &[] as &[u8],
                ReadError::IllegalFilename
            )))
        );
    }
    #[test]
    fn archive_correct() {
        assert_eq!(
            parse_archive(include_testfile!("archive_correct")),
            Ok((
                &[] as &[u8],
                types::Archive {
                    filesize: 74,
                    headersize: 74,
                    files: vec![
                        types::File {
                            header: types::Header {
                                filename_length: 28,
                                unused: 0,
                                is_file: true,
                                filesize: 10,
                                filename: String::from("ceci est un long text de 28 "),
                            },
                            absolute_offset: 94,
                            childs: vec![],
                        },
                        types::File {
                            header: types::Header {
                                filename_length: 28,
                                unused: 0,
                                is_file: true,
                                filesize: 11,
                                filename: String::from("ceci est un long text de 28b"),
                            },
                            absolute_offset: 104,
                            childs: vec![types::File {
                                header: types::Header {
                                    filename_length: 1,
                                    unused: 0,
                                    is_file: true,
                                    filesize: 1,
                                    filename: String::from("B"),
                                },
                                absolute_offset: 122,
                                childs: vec![],
                            },],
                        },
                    ],
                },
            ),)
        );
    }
}

mod write {
    use crate::types::write as types;
    use crate::write::*;
    #[test]
    fn file_from_path() {
        dbg!(path_to_file(path_testfile!("test_dir/")));
    }
}
