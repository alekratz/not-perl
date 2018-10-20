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

pub mod value {
    use std::fmt::{self, Formatter, Display};
    /// A common "constant value" structure used by all stages of compilation.
    #[derive(Debug, Clone, PartialEq)]
    pub enum Const {
        Int(i64),
        Float(f64),
        Str(String),
        Bool(bool),
    }
    
    impl Display for Const {
        fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
            match self {
                Const::Int(i) => write!(fmt, "{}", i),
                Const::Float(f) => write!(fmt, "{}", f),
                Const::Str(s) => write!(fmt, "{}", s),
                Const::Bool(b) => write!(fmt, "{}", b),
            }
        }
    }
}
