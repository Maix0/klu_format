use std::path::PathBuf;
pub fn pack(pack: PathBuf, archive: PathBuf) {
    let a = klu_core::write::Archive::from_path(pack);
    a.write_to_path(archive);
}

pub fn extract(extract: PathBuf, archive: PathBuf) {
    std::fs::create_dir_all(&extract).expect("Error while creating dirs");
    let mut a = klu_core::read::Archive::from_path(archive);
    a.release(extract);
}

pub fn list(archive: PathBuf) {
    let a = klu_core::read::Archive::from_path(archive);
    for l in &a.paths() {
        println!("{}", l);
    }
}
