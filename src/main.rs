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
    println!("Field 1 (some sort of header)");
    println!("  length: 0x{:04x}", length);
    println!("  value : {:02x?}", value);

    let length = read_u16(rest) as usize;
    let (value, rest) = read_n_byte(&rest[2..], length);
    println!("Field 2");
    println!("  length: 0x{:04x}", length);
    println!("  value : {}", str::from_utf8(value).unwrap());

    let length = read_u16(rest) as usize;
    let (value, rest) = read_n_byte(&rest[2..], length);
    println!("Field 3 (NCD name)");
    println!("  length: 0x{:04x}", length);
    println!("  value : {}", str::from_utf8(value).unwrap());

    let key = rest[0] as char;
    let length = read_u16(&rest[1..]) as usize;
    let (value, rest) = read_n_byte(&rest[3..], length);
    println!("Field 4 (model/part number)");
    println!("  key   : {}", key);
    println!("  length: 0x{:04x}", length);
    println!("  value : {}", str::from_utf8(value).unwrap());

    let key = rest[0] as char;
    let length = read_u16(&rest[1..]) as usize;
    let (value, rest) = read_n_byte(&rest[3..], length);
    println!("Field 5 (built date)");
    println!("  key   : {}", key);
    println!("  length: 0x{:04x}", length);
    println!("  value : {}", str::from_utf8(value).unwrap());

    let key = rest[0] as char;
    let length = read_u16(&rest[1..]) as usize;
    let (value, rest) = read_n_byte(&rest[3..], length);
    println!("Field 6 (built time)");
    println!("  key   : {}", key);
    println!("  length: 0x{:04x}", length);
    println!("  value : {}", str::from_utf8(value).unwrap());

    let key = rest[0] as char;
    let length = read_u32(&rest[1..]) as usize;
    let rest = &rest[5..];
    println!("Field 7 (data length)");
    println!("  key   : {}", key);
    println!("  length: 0x{:08x}", length);

    println!("");

    rest
}

fn dump(buffer : &[u8]) {
    /* 
     * Dump header
     */
    let body = dump_header(buffer);
    let body_len = body.len();

    /* 
     * Dump body
     */
    let mut cnt = 0;
    let mut zeros_cnt = 0;

    while cnt < body_len {
        // read 4 bytes
        let value = read_u32(&body[cnt..]);
        
        if value == 0 {
            zeros_cnt += 1;
        } else {
            zeros_cnt = 0;
        }

        if zeros_cnt < 4 {
            println!("{:08x} : {:08x}", cnt, value);
        } else if zeros_cnt == 4 {
            // Ignore contiguous zeros
            println!("...");
        }

        // next 4 bytes
        cnt += 4;
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
