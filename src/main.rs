use std::env;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::str;

fn read_u16(buffer : &[u8]) -> u16 {
    let mut ret : u16 = 0;
    for i in 0..2 {
        ret = (ret << 8) | (buffer[i] as u16);
    }
    ret
}

fn read_u32(buffer : &[u8]) -> u32 {
    let mut ret : u32 = 0;
    for i in 0..4 {
        ret = (ret << 8) | (buffer[i] as u32);
    }
    ret
}

fn read_n_byte(buffer : &[u8], n : usize) -> (&[u8], &[u8]) {
    (&buffer[..n], &buffer[n..])
}

/* 
 * For the format of the header information, see implementation of the file command.
 * https://github.com/file/file/blob/master/magic/Magdir/xilinx
 */
fn dump_header(buffer : &[u8]) -> &[u8] {
    let rest = buffer;

    let length = read_u16(rest) as usize;
    let (value, rest) = read_n_byte(&rest[2..], length);
    println!("Field 1");
    println!("  length: 0x{:04x}", length);
    println!("  value : {:02x?}", value);

    let length = read_u16(rest) as usize;
    let (value, rest) = read_n_byte(&rest[2..], length);
    println!("Field 2");
    println!("  length: 0x{:04x}", length);
    println!("  value : {}", str::from_utf8(value).unwrap());

    let length = read_u16(rest) as usize;
    let (value, rest) = read_n_byte(&rest[2..], length);
    println!("Field 3");
    println!("  length: 0x{:04x}", length);
    println!("  value : {}", str::from_utf8(value).unwrap());

    let key = rest[0] as char;
    let length = read_u16(&rest[1..]) as usize;
    let (value, rest) = read_n_byte(&rest[3..], length);
    println!("Field 4");
    println!("  key   : {}", key);
    println!("  length: 0x{:04x}", length);
    println!("  value : {}", str::from_utf8(value).unwrap());

    let key = rest[0] as char;
    let length = read_u16(&rest[1..]) as usize;
    let (value, rest) = read_n_byte(&rest[3..], length);
    println!("Field 5");
    println!("  key   : {}", key);
    println!("  length: 0x{:04x}", length);
    println!("  value : {}", str::from_utf8(value).unwrap());

    let key = rest[0] as char;
    let length = read_u16(&rest[1..]) as usize;
    let (value, rest) = read_n_byte(&rest[3..], length);
    println!("Field 6");
    println!("  key   : {}", key);
    println!("  length: 0x{:04x}", length);
    println!("  value : {}", str::from_utf8(value).unwrap());

    let key = rest[0] as char;
    let length = read_u32(&rest[1..]) as usize;
    let rest = &rest[5..];
    println!("Field 7");
    println!("  key   : {}", key);
    println!("  length: 0x{:04x}", length);

    rest
}

fn dump(buffer : &[u8]) {
    let rest = dump_header(buffer);

    /* 
     * Read body.
     */
    let mut cnt = 0;
    for byte in rest {
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
