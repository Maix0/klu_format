extern crate image;
extern crate klu;
use std::io::prelude::*;
fn main() {
    let mut archive = klu::read::Archive::from_path("./test/archive.klu");
    let mut reader = archive.get_virtual("in/testfile").unwrap();
    let mut buffer = [0; 8];
    reader
        .read(&mut buffer)
        .expect("Error while reading in VirtualFile");
    assert_eq!(buffer, [0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, 0x88]);
    buffer = [0; 8];
    let mut r1 = archive.get_virtual("in/backgroud.jpg").unwrap();
    r1.read(&mut buffer).unwrap();
    r1.seek(std::io::SeekFrom::Start(0)).unwrap();
    assert_eq!(buffer, r1.fill_buf().unwrap()[..8]);
    assert_eq!(buffer, [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46]);

    /* Retreave */
    r1.seek(std::io::SeekFrom::Start(0)).unwrap();
    let mut full_buf = Vec::new();
    let mut buffer = vec![0; 8 * 1024];
    let mut size = r1.read(&mut buffer).unwrap();
    while size > 0 {
        full_buf.extend_from_slice(&buffer[..size]);
        size = r1.read(&mut buffer).unwrap();
    }
    let img = image::load_from_memory(&full_buf[..]).unwrap();
    img.save("test/saved_img.jpg").unwrap();
}
