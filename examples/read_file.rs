extern crate klu_core;
use std::io::prelude::*;
#[cfg(feature = "virtual_fs")]
fn main() {
    let mut archive =
        klu_core::read::Archive::from_path("./test/archive.klu").expect("Unable to open archive");
    let mut reader = archive.get_virtual("archive/testfile").unwrap();
    let mut buffer = [0; 8];
    reader
        .read(&mut buffer)
        .expect("Error while reading in VirtualFile");
    assert_eq!(buffer, [0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, 0x88]);

    // Allow multiples virtuals files to be used at the same time;
    let mut buf_r1 = std::io::BufReader::new(archive.get_virtual("archive/image.jpg").unwrap());
    let mut buf_r2 = std::io::BufReader::new(archive.get_virtual("archive/image.jpg").unwrap());
    let mut buf1 = [0; 8];
    let mut buf2 = [0; 8];
    // Show that they are both used at the same time;
    assert_eq!(
        buf_r1.read(&mut buf1).unwrap(),
        buf_r2.read(&mut buf2).unwrap()
    );
    // Each virtuals files acts independently of the other, thus reading the same values;
    assert_eq!(buf1, buf2);

    let mut buf2 = [0; 5];
    assert_eq!(
        buf_r1.read(&mut buf1).unwrap(),
        buf_r2.read(&mut buf2).unwrap() + 3
    );
    // This works too
    assert_eq!(buf1[..5], buf2);
}

#[cfg(not(feature = "virtual_fs"))]
fn main() {
    println!("read_file exemple needs to run with the 'virtual_fs' feature");
    println!("cargo run --examples read_file --features virtual_fs");
}
