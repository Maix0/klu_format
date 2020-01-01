extern crate klu;

fn main() {
    let mut archive = klu::read::Archive::from_path("./test/archive.klu");
    assert_eq!(archive.path_exist("in/jesuisune.jpg"), true);
    assert_eq!(archive.path_exist("in/inexistant"), false);
    assert_eq!(archive.path_exist("path/that/do/not/exist/"), false);
}
