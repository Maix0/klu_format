extern crate klu_core;
fn main() {
    let mut archive =
        klu_core::read::Archive::from_path("./test/archive.klu").expect("Read archive error");
    archive.release("./test/out").expect("Release error");
}
