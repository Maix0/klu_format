extern crate klu_core;
fn main() {
    let archive = klu_core::write::Archive::from_path("./test/in").expect("Archive error");
    archive
        .write_to_path("./test/archive.klu")
        .expect("Archive write error");
    println!("'test/in' has been archive in 'test/archive.klu'")
}
