use std::env;
use std::io;
use std::io::prelude::*;
use std::fs::File;

fn dump(buffer : &Vec<u8>) {
    let mut cnt = 0;
    for byte in buffer {
        if cnt % 4 == 0 {
            print!("\n{:08x}", cnt);
        }
        print!(" {:02x}", byte);
        cnt += 1;
    }
    print!("\n");
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    let mut file = File::open(filename)?;
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;
    dump(&buffer);

    Ok(())
}
