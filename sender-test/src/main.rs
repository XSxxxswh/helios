use std::io::Read;
use std::os::unix::net::{UnixListener, UnixStream};
use std::usize;

fn main() {

    let path = "/tmp/helios.sock";
    let listener = UnixListener::bind(path).map_err(|e| {
        println!("bind error {:?}", e);
    }).unwrap();
    let (mut s, _) = listener.accept().map_err(|e| {
        println!("accept error {:?}", e);
    }).unwrap();
    loop {
        let mut lenb = [0u8; 8];
        s.read_exact(&mut lenb).unwrap();
        let len = usize::from_le_bytes(lenb);
        let mut buf = vec![0u8; len];
        s.read_exact(&mut buf).unwrap();
        let ev: Vec<solana_entry::entry::Entry> = bincode::deserialize(&buf).unwrap();
        for e in ev {
            println!("{:?}", e)
        }
    }
}
