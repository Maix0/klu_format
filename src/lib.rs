#[allow(dead_code)]
pub mod read;
pub mod write;
use std::path::PathBuf;

fn main() {
    /*
    let f = write::File {
        path: PathBuf::from("./test/in/testfile"),
        filesize: 0x15, // u64
        is_file: false,
        filename: String::from("testfile"),
        childs: Vec::new(),
    };
    */

    /*
    let f = write::File::from_path(PathBuf::from("./test/in/"));
    println!("{:2X?}", &*f.header());
    println!("{:#?}", f);
    */

    //let mut _a = write::Archive::from_path(PathBuf::from("./test/in"));
    //println!("{:#?}", a);
    //a.write_to_path(PathBuf::from("./test/in.klu"));

    let mut a = read::Archive::from_path(PathBuf::from("./test/in.klu"));
    //println!("{:#?}", a);
    a.release("./test/out");
}
