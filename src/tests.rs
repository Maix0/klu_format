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

#[macro_export]
macro_rules! dbg_eq {
    ($expr:expr, $rhs:expr) => {{
        let lhs = $expr;
        dbg!(&lhs);
        assert_eq!(lhs, $rhs);
    }};
}
mod read {
    use crate::read::{parse_archive, parse_header, parse_magic, NomReadErr, ReadError};
    use crate::types::read as types;
    #[test]
    fn magic_correct() {
        dbg_eq!(
            parse_magic(include_testfile!("magic_correct"), 0),
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
        dbg_eq!(
            parse_magic(include_testfile!("magic_wrong"), 0),
            Err(nom::Err::Error((
                &[0xDEu8, 0xAD, 0xBE, 0xEF] as &[u8],
                ReadError::WrongMagic(0xDEADBEEF, 0)
            ))) as NomReadErr<'_, &[u8]>
        );
    }
    #[test]
    fn header_correct() {
        dbg_eq!(
            parse_header(include_testfile!("header_correct"), 0),
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
        dbg_eq!(
            parse_header(include_testfile!("header_wrong"), 0),
            Err(nom::Err::Failure((
                &[] as &[u8],
                ReadError::IllegalFilename(0)
            )))
        );
    }
    #[test]
    fn archive_correct() {
        dbg_eq!(
            parse_archive(include_testfile!("archive_correct")),
            Ok((
                &[] as &[u8],
                types::Archive {
                    filesize: 123,
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
                                is_file: false,
                                filesize: 10,
                                filename: String::from("ceci est un long text de 28b"),
                            },
                            absolute_offset: 104,
                            childs: vec![types::File {
                                header: types::Header {
                                    filename_length: 1,
                                    unused: 0,
                                    is_file: true,
                                    filesize: 2,
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
    use std::path::PathBuf;
    #[test]
    fn file_from_path() {
        dbg_eq!(
            path_to_file(path_testfile!("test_dir/")).unwrap(),
            types::File {
                path: PathBuf::from(path_testfile!("test_dir/")),
                header: types::Header {
                    filename_length: "test_dir".as_bytes().len() as u8,
                    filename: String::from("test_dir"),
                    unused: 0,
                    is_file: false,
                    filesize: 24
                },
                childs: vec![
                    types::File {
                        path: PathBuf::from(path_testfile!("test_dir/empty_file")),
                        header: types::Header {
                            filename_length: "empty_file".as_bytes().len() as u8,
                            filename: String::from("empty_file"),
                            unused: 0,
                            is_file: true,
                            filesize: 0
                        },
                        childs: vec![],
                    },
                    types::File {
                        path: PathBuf::from(path_testfile!("test_dir/full_file")),
                        header: types::Header {
                            filename_length: "full_file".as_bytes().len() as u8,
                            filename: String::from("full_file"),
                            unused: 0,
                            is_file: true,
                            filesize: 16
                        },
                        childs: vec![]
                    }
                ]
            }
        );
    }
    #[test]
    fn multi_archive_from_path() {
        dbg_eq!(
            multi_path_to_archive(path_testfile!("test_dir/")).unwrap(),
            types::Archive {
                base_path: PathBuf::from(path_testfile!("test_dir/")),
                childs: vec![
                    types::File {
                        path: PathBuf::from(path_testfile!("test_dir/empty_file")),
                        header: types::Header {
                            filename_length: "empty_file".as_bytes().len() as u8,
                            filename: String::from("empty_file"),
                            unused: 0,
                            is_file: true,
                            filesize: 0
                        },
                        childs: vec![],
                    },
                    types::File {
                        path: PathBuf::from(path_testfile!("test_dir/full_file")),
                        header: types::Header {
                            filename_length: "full_file".as_bytes().len() as u8,
                            filename: String::from("full_file"),
                            unused: 0,
                            is_file: true,
                            filesize: 16
                        },
                        childs: vec![]
                    }
                ]
            }
        );
    }
    #[test]
    fn archive_from_path() {
        dbg_eq!(
            single_path_to_archive(path_testfile!("test_dir/")).unwrap(),
            types::Archive {
                base_path: PathBuf::from(path_testfile!("test_dir/")),
                childs: vec![types::File {
                    path: PathBuf::from(path_testfile!("test_dir/")),
                    header: types::Header {
                        filename_length: "test_dir".as_bytes().len() as u8,
                        filename: String::from("test_dir"),
                        unused: 0,
                        is_file: false,
                        filesize: 24
                    },
                    childs: vec![
                        types::File {
                            path: PathBuf::from(path_testfile!("test_dir/empty_file")),
                            header: types::Header {
                                filename_length: "empty_file".as_bytes().len() as u8,
                                filename: String::from("empty_file"),
                                unused: 0,
                                is_file: true,
                                filesize: 0
                            },
                            childs: vec![],
                        },
                        types::File {
                            path: PathBuf::from(path_testfile!("test_dir/full_file")),
                            header: types::Header {
                                filename_length: "full_file".as_bytes().len() as u8,
                                filename: String::from("full_file"),
                                unused: 0,
                                is_file: true,
                                filesize: 16
                            },
                            childs: vec![]
                        }
                    ]
                }]
            }
        );
    }
}
