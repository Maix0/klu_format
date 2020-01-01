extern crate klu;
fn main() {
    let mut archive = klu::read::Archive::from_path("./test/archive.klu");
    archive.release("./test/out");
}
