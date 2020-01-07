extern crate image;
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

    let mut bufR = std::io::BufReader::new(archive.get_virtual("archive/image.jpg").unwrap());
    let image = image::load(bufR, image::ImageFormat::JPEG).unwrap();
    match image {
        image::DynamicImage::ImageRgb8(img) => assert_eq!(img.dimensions(), (400, 345)),
        _ => panic!("img format"),
    }
    let mut bufR2 = std::io::BufReader::new(archive.get_virtual("archive/image.jpg").unwrap());
}
#[cfg(not(feature = "virtual_fs"))]
fn main() {
    println!("read_file exemple needs to run with the 'virtual_fs' feature");
    println!("cargo run --examples read_file --features virtual_fs");
}
