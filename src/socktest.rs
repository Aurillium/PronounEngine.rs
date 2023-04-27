use std::os::unix::net::UnixStream;
use std::path::Path;
use std::io::prelude::*;

fn main() {
    let socket = Path::new("/run/user/1000/pronoun_engine.sock");

    // Connect to socket
    let mut stream = UnixStream::connect(&socket).unwrap();
    stream.write_all(b"Hello@").unwrap();
    stream.flush().unwrap();

    return;
}
