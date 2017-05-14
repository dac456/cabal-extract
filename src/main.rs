#![crate_name = "cabal_extract"]

extern crate byteorder;
extern crate bmp;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::string::String;

use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};

use bmp::{Pixel, Image};

struct Texture {
    name: String,
    bitmap_idx: usize,
    colour_idx: usize,
    pixel_size: f32
}

struct Bitmap {
    name: String,
    width: u32,
    height: u32,
    data: Vec<u8>
}

struct Palette {
    r: [u8; 256],
    g: [u8; 256],
    b: [u8; 256],
}

fn read_string<T: ReadBytesExt>(buf: &mut T, len: u32) -> String {
    let mut str = Vec::new();
    for _ in 0..len {
        match buf.read_u8() {
            Ok(b) => str.push(b),
            Err(err) => println!("error {}", err)
        }
    }

    return String::from_utf8_lossy(str.as_slice()).into_owned();
}

fn read_header<T: ReadBytesExt>(mut buf: &mut T) -> Result<(u16, u32), std::io::Error> {
    let id = buf.read_u16::<LittleEndian>()? as u16;
    let tag_size = buf.read_u32::<LittleEndian>()? as u32;

    Ok((id, tag_size))
}

fn read_file_info<T: ReadBytesExt>(mut buf: &mut T) {
    println!("got name: {}", read_string(&mut buf, 32));
    println!("got author: {}", read_string(&mut buf, 32));
    println!("got date: {}", read_string(&mut buf, 32));
    println!("got version: {}", read_string(&mut buf, 16));
}

fn read_palette_data<T: ReadBytesExt>(mut buf: &mut T) -> Palette {
    let mut p = Palette { r: [0; 256], g: [0; 256], b: [0; 256] };

    for i in 0..256 {
        p.r[i] = buf.read_u8().unwrap();
        p.g[i] = buf.read_u8().unwrap();
        p.b[i] = buf.read_u8().unwrap();
    }

    return p
}

fn read_bitmap_data<T: ReadBytesExt>(mut buf: &mut T) -> Bitmap {
    let hdr_size = buf.read_u16::<LittleEndian>().unwrap();
    let name = read_string(&mut buf, 14);
    let x_len = buf.read_u16::<LittleEndian>().unwrap() as u32;
    let y_len = buf.read_u16::<LittleEndian>().unwrap() as u32;
    let flags = buf.read_u16::<LittleEndian>().unwrap();

    let sz: u32 = x_len as u32 * y_len as u32;
    let mut data = Vec::new();

    for _ in 0..sz {
        data.push(buf.read_u8().unwrap());
    }

    let bmp = Bitmap { name: name, width: x_len, height: y_len, data: data };

    return bmp
}

fn read_texture_data<T: ReadBytesExt>(mut buf: &mut T) -> Texture {
    let mut texture = Texture { name: String::from(""), bitmap_idx: 0, colour_idx: 0, pixel_size: 0.0 };
    texture.name = read_string(&mut buf, 32);
    texture.bitmap_idx = buf.read_u16::<LittleEndian>().unwrap() as usize;
    texture.colour_idx = buf.read_u16::<LittleEndian>().unwrap() as usize;
    texture.pixel_size = buf.read_f32::<LittleEndian>().unwrap();

    return texture
}


fn main() {
    let mut file = File::open("C:\\Users\\Daniel\\Downloads\\alien-cabal\\AlienCabal\\acabal.gob").unwrap();

    let mut data = Vec::new();
    file.read_to_end(&mut data);
    let mut buffer = Cursor::new(&data);

    let mut palettes = Vec::new();
    let mut bitmaps = Vec::new();
    let mut textures = Vec::new();

    loop {
        let header = match read_header(&mut buffer){
            Ok(x) => x,
            Err(e) => break
        };

        match header.0 {
            2 => {
                let p = read_palette_data(&mut buffer);
                palettes.push(p);
            },
            14 => {
                let b = read_bitmap_data(&mut buffer);
                bitmaps.push(b);
            },
            15 => {
                let t = read_texture_data(&mut buffer);
                textures.push(t);
            },
            20 => read_file_info(&mut buffer),
            _ => {
                println!("unknown id {}", header.0);
                buffer.seek(SeekFrom::Current((header.1 as i64) - 6));
            }
        }
    }

    for (i, t) in textures.iter().enumerate() {
        let bmp = &bitmaps[t.bitmap_idx];
        let mut img = Image::new(bmp.width, bmp.height);

        for (x, y) in img.coordinates() {
            let pixel = bmp.data[((y * bmp.width) + x) as usize];
            let r = palettes[t.colour_idx].r[pixel as usize];
            let g = palettes[t.colour_idx].g[pixel as usize];
            let b = palettes[t.colour_idx].b[pixel as usize];

            img.set_pixel(x, y, Pixel { r: r, g: g, b: b });
        }

        let _ = img.save(&format!("out/{}.bmp", i)[..]);
    }
}
