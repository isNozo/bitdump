use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::str;

/* 
 * Configuration Packet
 * 
 * Type1:
 * +-------------+---------+------------------+----------+-------------+
 * | Header Type | Opecode | Register Address | Reserved | Word Count  |
 * | [31:29]     | [28:27] | [26:13]          | [12:11]  | [10:0]      |
 * +-------------+---------+------------------+----------+-------------+
 * | 001         | xx      | RRRRRRRRRxxxxx   | RR       | xxxxxxxxxxx |
 * +-------------+---------+------------------+----------+-------------+
 * 
 * Type2:
 * +-------------+---------+-----------------------------+
 * | Header Type | Opecode | Word Count                  |
 * | [31:29]     | [28:27] | [26:0]                      |
 * +-------------+---------+-----------------------------+
 * | 010         | RR      | xxxxxxxxxxxxxxxxxxxxxxxxxxx |
 * +-------------+---------+-----------------------------+
 */

const MASK_HEADER_TYPE: u32 = 0b111 << 29;
const HEADER_TYPE1: u32     = 0b001 << 29;
const HEADER_TYPE2: u32     = 0b010 << 29;

const MASK_OP:u32   = 0b11 << 27;
const OP_NOP: u32   = 0b00 << 27;
const OP_READ: u32  = 0b01 << 27;
const OP_WRITE: u32 = 0b10 << 27;

const MASK_REG: u32    = 0b11111 << 13;
const REG_CRC: u32     = 0b00000 << 13;
const REG_FAR: u32     = 0b00001 << 13;
const REG_FDRI: u32    = 0b00010 << 13;
const REG_FDRO: u32    = 0b00011 << 13;
const REG_CMD: u32     = 0b00100 << 13;
const REG_CTL0: u32    = 0b00101 << 13;
const REG_MASK: u32    = 0b00110 << 13;
const REG_STAT: u32    = 0b00111 << 13;
const REG_LOUT: u32    = 0b01000 << 13;
const REG_COR0: u32    = 0b01001 << 13;
const REG_MFWR: u32    = 0b01010 << 13;
const REG_CBC: u32     = 0b01011 << 13;
const REG_IDCODE: u32  = 0b01100 << 13;
const REG_AXSS: u32    = 0b01101 << 13;
const REG_COR1: u32    = 0b01110 << 13;
const REG_WBSTAR: u32  = 0b10000 << 13;
const REG_TIMER: u32   = 0b10001 << 13;
const REG_BOOTSTS: u32 = 0b10110 << 13;
const REG_CTL1: u32    = 0b11000 << 13;
const REG_BSPI: u32    = 0b11111 << 13;

const MASK_WORDCOUNT_TYPE1: u32 = 0b1111111111;
const MASK_WORDCOUNT_TYPE2: u32 = 0b111111111111111111111111111;

fn read_u16(buffer: &[u8]) -> u16 {
    let mut ret: u16 = 0;
    for i in 0..2 {
        ret = (ret << 8) | (buffer[i] as u16);
    }
    ret
}

fn read_u32(buffer: &[u8]) -> u32 {
    let mut ret: u32 = 0;
    for i in 0..4 {
        ret = (ret << 8) | (buffer[i] as u32);
    }
    ret
}

fn read_n_byte(buffer: &[u8], n: usize) -> (&[u8], &[u8]) {
    (&buffer[..n], &buffer[n..])
}

/*
 * For the format of the header information, see implementation of the file command.
 * https://github.com/file/file/blob/master/magic/Magdir/xilinx
 */
fn dump_header(buffer: &[u8]) -> &[u8] {
    println!("== Header Information ==");

    let rest = buffer;

    // Field 1 (some sort of header)
    let length = read_u16(rest) as usize;
    let (_, rest) = read_n_byte(&rest[2..], length);

    // Field 2
    let length = read_u16(rest) as usize;
    let (_, rest) = read_n_byte(&rest[2..], length);

    // Field 3 (NCD name)
    let length = read_u16(rest) as usize;
    let (value, rest) = read_n_byte(&rest[2..], length);
    println!("NCD name          : {}", str::from_utf8(value).unwrap());

    // Field 4 (model/part number)
    let length = read_u16(&rest[1..]) as usize;
    let (value, rest) = read_n_byte(&rest[3..], length);
    println!("model/part number : {}", str::from_utf8(value).unwrap());

    // Field 5 (built date)
    let length = read_u16(&rest[1..]) as usize;
    let (value, rest) = read_n_byte(&rest[3..], length);
    println!("built date        : {}", str::from_utf8(value).unwrap());

    // Field 6 (built time)
    let length = read_u16(&rest[1..]) as usize;
    let (value, rest) = read_n_byte(&rest[3..], length);
    println!("built time        : {}", str::from_utf8(value).unwrap());

    // Field 7 (data length)
    let length = read_u32(&rest[1..]) as usize;
    let rest = &rest[5..];
    println!("data length       : 0x{:08x}", length);

    rest
}

fn dump_n_word(buffer: &[u8], n: usize, offset: usize) {
    let mut bytes_cnt = offset;
    let mut duplicates_cnt = 0;
    let mut prev_value = 0;

    while bytes_cnt < offset + n {
        // read 4 bytes
        let value = read_u32(&buffer[bytes_cnt..]);
        if value == prev_value {
            duplicates_cnt += 1;
        } else {
            duplicates_cnt = 0;
        }
        prev_value = value;
        bytes_cnt += 4;

        if duplicates_cnt < 4 {
            println!("{:08x} : {:08x} ", bytes_cnt-4, value);
        } else if duplicates_cnt == 4 {
            // Ignore duplicate bytes
            println!("*");
        }
    }
}

fn dump(buffer: &[u8]) {
    /*
     * Dump header
     */
    let body = dump_header(buffer);
    let body_len = body.len();

    /*
     * Dump body
     */
    println!("\n== Configuration Data ==");

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
        prev_value = value;
        bytes_cnt += 4;

        if duplicates_cnt < 4 {
            print!("{:08x} : {:08x} ", bytes_cnt-4, value);

            let header_type = value & MASK_HEADER_TYPE;
            match header_type {
                HEADER_TYPE1 => {
                    print!("Type1 ");
                    let op = value & MASK_OP;
                    match op {
                        OP_NOP   => { println!("NOOP"); continue; },
                        OP_READ  => print!("READ  "),
                        OP_WRITE => print!("WRITE "),
                        _        => print!("InvOP "),
                    }

                    let reg = value & MASK_REG;
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
                        _           => print!("InvREG  "),
                    }

                    let word_cnt = value & MASK_WORDCOUNT_TYPE1;
                    println!("word_cnt={}", word_cnt);
                    dump_n_word(&body, word_cnt as usize, bytes_cnt);
                    bytes_cnt += 4 * word_cnt as usize;

                    continue;
                }
                HEADER_TYPE2 => {
                    print!("Type2 ");

                    let word_cnt = value & MASK_WORDCOUNT_TYPE2;
                    println!("word_cnt={}", word_cnt);
                    dump_n_word(&body, word_cnt as usize, bytes_cnt);
                    bytes_cnt += 4 * word_cnt as usize;
                    continue;
                }
                _ => (),
            }
            println!("");
        } else if duplicates_cnt == 4 {
            // Ignore duplicate bytes
            println!("*");
        }
    }

    println!("{:08x} bytes were read.", bytes_cnt);
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
