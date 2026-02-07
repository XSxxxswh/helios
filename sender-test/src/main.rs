
use std::{
    fs,
    io::{self, Read},
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
    time::Duration,
};

fn read_frame(stream: &mut UnixStream) -> io::Result<Vec<u8>> {
    let mut lenb = [0u8; 8];
    stream.read_exact(&mut lenb)?;               // может вернуть UnexpectedEof
    let len = u64::from_le_bytes(lenb) as usize; // фиксируем протокол как u64

    const MAX: usize = 64 * 1024 * 1024; // 64MB — подстрой под себя
    if len == 0 || len > MAX {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "bad frame length"));
    }

    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;
    Ok(buf)
}

fn handle_client(mut s: UnixStream) -> io::Result<()> {
    loop {
        match read_frame(&mut s) {
            Ok(buf) => {
                // bincode может упасть на битых данных — не валим весь процесс
                match bincode::deserialize::<Vec<solana_entry::entry::Entry>>(&buf) {
                    Ok(ev) => {
                        for e in ev {
                            println!("{:?}", e);
                        }
                    }
                    Err(e) => eprintln!("bincode deserialize error: {e}"),
                }
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                eprintln!("client disconnected (EOF)");
                return Ok(());
            }
            Err(e) => {
                eprintln!("read error: {e}");
                return Err(e);
            }
        }
    }
}

fn main() -> io::Result<()> {
    let path = "/tmp/helios.sock";

    // Критично: удалить старый сокет-файл (stale) перед bind
    if Path::new(path).exists() {
        let _ = fs::remove_file(path);
    }

    let listener = UnixListener::bind(path)?;
    // чтобы не зависал accept навечно на shutdown сценариях (не обязательно)
    // listener.set_nonblocking(true)?;

    loop {
        match listener.accept() {
            Ok((s, _addr)) => {
                eprintln!("client connected");
                if let Err(e) = handle_client(s) {
                    eprintln!("client handler error: {e}");
                }
                // после дисконнекта идём accept следующего
            }
            Err(e) => {
                eprintln!("accept error: {e}");
                std::thread::sleep(Duration::from_millis(200));
            }
        }
    }
}

