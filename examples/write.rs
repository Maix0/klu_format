extern crate klu;
fn main() {
    let archive = klu::write::Archive::from_path("./test/in");
    archive.write_to_path("./test/archive.klu");
    println!("'test/in' has been archive in 'test/archive.klu'")
}
