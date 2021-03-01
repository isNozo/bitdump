use std::env;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::str;

const HEADER_TYPE1 : u32 = 0b001;
const HEADER_TYPE2 : u32 = 0b010;

const OP_NOP   : u32 = 0b00;
const OP_READ  : u32 = 0b01;
const OP_WRITE : u32 = 0b10;

const REG_CRC : u32 = 0b00000;
const REG_FAR : u32 = 0b00001;
const REG_FDRI : u32 = 0b00010;
const REG_FDRO : u32 = 0b00011;
const REG_CMD : u32 = 0b00100;
const REG_CTL0 : u32 = 0b00101;
const REG_MASK : u32 = 0b00110;
const REG_STAT : u32 = 0b00111;
const REG_LOUT : u32 = 0b01000;
const REG_COR0 : u32 = 0b01001;
const REG_MFWR : u32 = 0b01010;
const REG_CBC : u32 = 0b01011;
const REG_IDCODE : u32 = 0b01100;
const REG_AXSS : u32 = 0b01101;
const REG_COR1 : u32 = 0b01110;
const REG_WBSTAR : u32 = 0b10000;
const REG_TIMER : u32 = 0b10001;
const REG_BOOTSTS : u32 = 0b10110;
const REG_CTL1 : u32 = 0b11000;
const REG_BSPI : u32 = 0b11111;

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

fn get_bitfield(value : u32, start_bit : u8, end_bit : u8) -> u32 {
    let length = end_bit - start_bit + 1;
    let mask = !(!0 << length);
    let ret = (value >> start_bit) & mask;
    ret
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
    let mut bytes_cnt = 0;
    let mut duplicates_cnt = 0;
    let mut prev_value = 0;

    while bytes_cnt < body_len {
        // read 4 bytes
        let value = read_u32(&body[bytes_cnt..]);
        
        if value == prev_value {
            duplicates_cnt += 1;
        } else {
            duplicates_cnt = 0;
        }

        if duplicates_cnt < 4 {
            print!("{:08x} : {:032b} ", bytes_cnt, value);

            let header_type = get_bitfield(value, 29, 31);
            match header_type {
                HEADER_TYPE1 => {
                    print!("TYPE   1 ");
                    let op = get_bitfield(value, 27, 28);
                    match op {
                        OP_NOP   => print!("NOP   "),
                        OP_READ  => print!("READ  "),
                        OP_WRITE => print!("WRITE "),
                        _        => print!("InvOP ")
                    }

                    let reg = get_bitfield(value, 13, 26);
                    print!("({:05b}) ", reg);
                    match reg {
                        REG_CRC     => print!("CRC     "),
                        REG_FAR     => print!("FAR     "),
                        REG_FDRI    => print!("FDRI    "),
                        REG_FDRO    => print!("FDRO    "),
                        REG_CMD     => print!("CMD     "),
                        REG_CTL0    => print!("CTL0    "),
                        REG_MASK    => print!("MASK    "),
                        REG_STAT    => print!("STAT    "),
                        REG_LOUT    => print!("LOUT    "),
                        REG_COR0    => print!("COR0    "),
                        REG_MFWR    => print!("MFWR    "),
                        REG_CBC     => print!("CBC     "),
                        REG_IDCODE  => print!("IDCODE  "),
                        REG_AXSS    => print!("AXSS    "),
                        REG_COR1    => print!("COR1    "),
                        REG_WBSTAR  => print!("WBSTAR  "),
                        REG_TIMER   => print!("TIMER   "),
                        REG_BOOTSTS => print!("BOOTSTS "),
                        REG_CTL1    => print!("CTL1    "),
                        REG_BSPI    => print!("BSPI    "),
                        _           => print!("Inv reg "),
                    }

                    let word_cnt = get_bitfield(value, 0, 10);
                    print!("word_cnt={} ", word_cnt);
                }
                HEADER_TYPE2 => {
                    print!("TYPE   2 ");
                }
                _ => print!("TYPE Inv ")
            }
            println!("");

        } else if duplicates_cnt == 4 {
            // Ignore duplicate bytes
            println!("*");
        }

        prev_value = value;
        bytes_cnt += 4;
    }

    println!("{:08x} bytes are read.", bytes_cnt);
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
