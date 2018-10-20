use std::{
    io::{self, Read},
    path::Path,
    fs::File,
};

pub fn read_file(path: impl AsRef<Path>) -> io::Result<String> {
    let mut file = File::open(path.as_ref())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}
