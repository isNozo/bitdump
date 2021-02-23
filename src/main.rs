use std::env;
use std::io;
use std::io::prelude::*;
use std::fs::File;

fn read_u16(buffer : &[u8]) -> u16 {
    let mut ret : u16 = 0;
    for i in 0..2 {
        ret = (ret << 8) | (buffer[i] as u16);
    }
    ret
}

#[allow(dead_code)]
fn read_u32(buffer : &[u8]) -> u32 {
    let mut ret : u32 = 0;
    for i in 0..4 {
        ret = (ret << 8) | (buffer[i] as u32);
    }
    ret
}

fn read_field(buffer : &[u8]) -> usize {
    let length = read_u16(buffer) as usize;
    println!("  length: 0x{:04x}", length);
    print!("  value : ");
    for i in 0..length {
        print!("{}", buffer[2+i] as char);
    }
    print!("\n");
    length
}

fn dump(buffer : &Vec<u8>) {
    let mut offset = 0;

    /* 
     * Read header. This is referencing implementation of file command.
     * https://github.com/file/file/blob/master/magic/Magdir/xilinx
     */
    println!("Field 1");
    let length = read_field(&buffer[offset..]) as usize;
    offset += 2+length;

    println!("Field 2");
    let length = read_field(&buffer[offset..]) as usize;
    offset += 2+length;
    
    println!("Field 3");
    let length = read_field(&buffer[offset..]) as usize;
    offset += 2+length;

    println!("Field 4");
    println!("  key   : {}", buffer[offset] as char);
    offset += 1;
    let length = read_field(&buffer[offset..]) as usize;
    offset += 2+length;

    println!("Field 5");
    println!("  key   : {}", buffer[offset] as char);
    offset += 1;
    let length = read_field(&buffer[offset..]) as usize;
    offset += 2+length;

    println!("Field 6");
    println!("  key   : {}", buffer[offset] as char);
    offset += 1;
    let length = read_field(&buffer[offset..]) as usize;
    offset += 2+length;

    println!("Field 7");
    println!("  key   : {}", buffer[offset] as char);
    offset += 1;
    let length = read_u32(buffer) as usize;
    println!("  length: 0x{:08x}", length);
    offset += 4;

    let mut cnt = 0;
    for byte in &buffer[offset..] {
        if cnt % 4 == 0 {
            print!("\n{:08x}", offset+cnt);
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
