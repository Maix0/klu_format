use klu_v2::types::read;

fn main() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/testsdata/archive_correct");
    let archive = read::ArchiveFile::from_file(&path).unwrap().unwrap();
    let err = archive.extract(concat!(env!("CARGO_MANIFEST_DIR"), "/testsdata/extract/"));
    if let Err(e) = err {
        println!("Error: {}", e);
    } else {
        println!("Extracted!");
    }
}
