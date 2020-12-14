use std::io;
use std::path::Path;

pub fn str_prefix_io(pre: &str, err: io::Error) -> io::Error {
    io::Error::new(err.kind(), format!("{}: {}", pre, err))
}

pub fn path_prefix_io(pre: &Path, err: io::Error) -> io::Error {
    str_prefix_io(pre.to_str().unwrap(), err)
}
