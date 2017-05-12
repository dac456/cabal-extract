#![crate_name = "cabal_extract"]

extern crate byteorder;

use std::fs::File;
use std::io::Read;
use std::string::String;

use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};

fn read_string<T: ReadBytesExt>(buf: &mut T, len: u32) -> String {
    let mut str = Vec::new();
    for _ in 0..len {
        let b = buf.read_u8().unwrap();
        str.push(b)
    }

    //let mut date = vec![0u8, 32];
    //buffer.read_exact(date.as_mut_slice()).unwrap();

    return String::from_utf8_lossy(str.as_slice()).into_owned();
}

fn read_file_info<T: ReadBytesExt>(mut buf: &mut T) {
    let id = buf.read_u16::<LittleEndian>().unwrap();
    let tag_size = buf.read_u32::<LittleEndian>().unwrap();

    println!("got name: {}", read_string(&mut buf, 32));
    println!("got author: {}", read_string(&mut buf, 32));
    println!("got date: {}", read_string(&mut buf, 32));
    println!("got version: {}", read_string(&mut buf, 16));
}

fn main() {
    let mut file = File::open("C:\\Users\\Daniel\\Downloads\\alien-cabal\\AlienCabal\\acabal.gob").unwrap();

    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();
    let mut buffer = Cursor::new(&data);

    read_file_info(&mut buffer);
}
