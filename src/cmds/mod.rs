use std::path::PathBuf;
pub fn pack(pack: PathBuf, archive: PathBuf) -> Result<(), String> {
    let a = klu_core::write::Archive::from_path(pack);
    match a {
        Ok(_) => {}
        Err(e) => {
            return Err(format!(
                "Error while reading files for archive creation: \n{}",
                e
            ));
        }
    }
    match a.unwrap().write_to_path(archive) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!("Error while packing archive: \n{}", e));
        }
    };
    Ok(())
}

pub fn extract(extract: PathBuf, archive: PathBuf) -> Result<(), String> {
    match std::fs::create_dir_all(&extract) {
        Ok(_) => (),
        Err(e) => {
            return Err(format!("Error while creating Extract base dir:\n{}", e));
        }
    };
    let a = klu_core::read::Archive::from_path(archive);
    match a {
        Ok(_) => {}
        Err(e) => {
            return Err(format!("Error while reading archive from file: \n{}", e));
        }
    }
    match a.unwrap().release(extract) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!("Error while extracting archive: \n{}", e));
        }
    };
    Ok(())
}

pub fn list(archive: PathBuf) -> Result<(), String> {
    let a = klu_core::read::Archive::from_path(archive);
    match a {
        Ok(_) => {}
        Err(e) => {
            return Err(format!("Error while reading archive from file: \n{}", e));
        }
    }
    for l in &a.unwrap().paths() {
        println!("{}", l);
    }
    Ok(())
}
