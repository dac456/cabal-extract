#![crate_name = "cabal_extract"]
#![feature(step_by)]

extern crate byteorder;
extern crate bmp;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::string::String;

use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};

use bmp::{Pixel, Image};

mod postprocess;
use postprocess::superxbr;

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

struct Level {
    rebuild_zone: u16
}

fn read_string<T: ReadBytesExt>(buf: &mut T, len: usize) -> String {
    let mut str = vec![' ' as u8; len];
    for i in 0..len {
        match buf.read_u8() {
            Ok(b) => str[i] = b,
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
    println!("GOB Info:");
    println!(" -- Name: {}", read_string(&mut buf, 32));
    println!(" -- Author: {}", read_string(&mut buf, 32));
    println!(" -- Date: {}", read_string(&mut buf, 32));
    println!(" -- VEDIT Version: {}", read_string(&mut buf, 16));
}

fn read_palette_data<T: ReadBytesExt>(mut buf: &mut T) -> Palette {
    let mut p = Palette { r: [0; 256], g: [0; 256], b: [0; 256] };

    for i in 0..256 {
        p.r[i] = buf.read_u8().unwrap();
    }
    for i in 0..256 {
        p.g[i] = buf.read_u8().unwrap();
    }
    for i in 0..256 {
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

fn read_level_data<T: ReadBytesExt>(mut buf: &mut T) -> Level {
    let mut level = Level { rebuild_zone: 0 };
//    let hdr_size = buf.read_u16::<LittleEndian>().unwrap();

    return level
}


fn main() {
    let mut file = File::open("acabal.gob").unwrap();

    let mut data = Vec::new();
    file.read_to_end(&mut data);
    let mut buffer = Cursor::new(&data);

    let mut palettes = Vec::new();
    let mut bitmaps = Vec::new();
    let mut textures = Vec::new();
    let mut levels = Vec::new();

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
            33 => println!("old level data"),
            40 => {
                let _ = buffer.seek(SeekFrom::Current((header.1 as i64) - 6));
                let l = read_level_data(&mut buffer);
                levels.push(l)
            },
            _ => {
                println!("unknown id {}", header.0);
                let _ = buffer.seek(SeekFrom::Current((header.1 as i64) - 6));
            }
        }
    }

    println!("Found {} bitmaps", bitmaps.len());
    println!("Found {} palettes", palettes.len());
    println!("Found {} textures", textures.len());
    println!("Found {} levels", levels.len());

    for (i, t) in textures.iter().enumerate() {
        let bmp = &bitmaps[t.bitmap_idx];
        let mut img = Image::new(bmp.width*2, bmp.height*2);

        let mut to_scale = vec![0u32; (bmp.width*bmp.height) as usize];
        let mut scaled = vec![0u32; (bmp.width*bmp.height*4) as usize];

        for y in 0..bmp.height {
            for x in 0..bmp.width {
                let pixel = bmp.data[((y * bmp.width) + x) as usize];
                let r = palettes[t.colour_idx].r[pixel as usize];
                let g = palettes[t.colour_idx].g[pixel as usize];
                let b = palettes[t.colour_idx].b[pixel as usize];

                let p: u32 = (r as u32) | ((g as u32) << 8) | ((b as u32) << 16) | ((255 as u32) << 24);
                to_scale[((y * bmp.width) + x) as usize] = p;

                //            img.set_pixel(x, y, Pixel { r: r, g: g, b: b });
            }
        }

        superxbr::scale(to_scale.as_mut_slice(), scaled.as_mut_slice(), bmp.width as i32, bmp.height as i32);

        for (x, y) in img.coordinates() {
            let idx: usize = ((y * bmp.width) + x) as usize;
            let r: u8 = (scaled[idx] & 0xff) as u8;
            let g: u8 = ((scaled[idx] >> 8) & 0xff) as u8;
            let b: u8 = ((scaled[idx] >> 16) & 0xff) as u8;

            img.set_pixel(x, y, Pixel { r: r, g: g, b: b });
        }

        println!("Saving texture {}...", i);
        let _ = img.save(&format!("out/{}.bmp", i)[..]);
    }
}
