use std::fs::File;
use std::io::prelude::*;
use std::path::Path;


fn main() {
    println!("Hello, world!");
    create_file()
}

fn create_file(){
    let path: &Path = Path::new("test.desk");
    let display = path.display();
    println!("{}", display);

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file
    };

    // Écrit la chaîne `LOREM_IPSUM` dans `file`, renvoie `io::Result<()>`
    match file.write_all("Hello, world!".as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}