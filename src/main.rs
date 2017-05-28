#![crate_name = "cabal_extract"]
#![feature(step_by)]

extern crate byteorder;
extern crate bmp;

use std::fs;
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

#[derive(Default)]
struct Vertex {
    flags: u16, // word
    x: f32,
    y: f32,
    z: f32,
    sprite_index: i16,
    noise_index: i16,
    lifeform_index: i16,
    orientation: i16,
    group_index: i16,
    item_index: i16,
    trigger_index: i16,
}

#[derive(Default)]
struct Polygon {
    flags: u16, // word
    polygon_link: i16,
    texture_index: i16,
    bmp_x: [f32; 2],
    bmp_y: [f32; 2],
    indices: [i16; 8],
    animation_index: i16,
    texture_dx: f32,
    texture_dy: f32,
    motion_index: i16,
    group_index: i16,
    texture_x_offset: f32,
    texture_y_offset: f32,
    damage_animation_index: i16
}

#[derive(Default)]
struct Trigger {
    name: String, // 32 chars
    flags: u16, // word
    radius: f32,
    height: f32,
    item_type: i16,
    sound_index: i16
}

#[derive(Default)]
struct Motion {
    name: String, // 32 chars
    flags: u16, // word
    dx: f32,
    dy: f32,
    dz: f32,
    num_steps: i16,
    return_delay: i16,
    trigger_index: i16,
    sound_index_start: i16,
    sound_index_run: i16,
    sound_index_stop: i16
}

#[derive(Default)]
struct Group {
    name: String // 32 chars
}

#[derive(Default)]
struct Mark {
    name: String, // 32 chars
    x: i32,
    y: i32,
    z: i32,
    rotation: f64,
    elevation: f64
}

#[derive(Default)]
struct Level {
    name: String, // 32 chars
    author: String, // 32 chars
    num_vertices: i16,
    num_polygons: i16,
    num_triggers: i16,
    num_motions: i16,
    num_groups: i16,
    num_marks: i16,
    last_mark: i16,
    view_home_rotation: f64,
    view_home_elevation: f64,
    view_home_x: i32,
    view_home_y: i32,
    view_home_z: i32,
    view_rotation: f64,
    view_elevation: f64,
    view_x: i32,
    view_y: i32,
    view_z: i32,
    view_delta: i32,
    view_depth: i32,
    view_width: i32,
    view_height: i32,
    grid_x: i32,
    grid_y: i32,
    grid_z: i32,
    grid_rotation: f64,
    grid_elevation: f64,
    grid_delta: i32,
    grid_spacing: i32,
    grid_size: i32,
    maintain_grid_dist: u16, // bool
    lock_view_to_grid: u16, // bool
    snap_to_grid: u16, // bool
    flags: u16, // word
    bkg_bitmap_index: i16,
    sol_bitmap_index: i16,
    eol_bitmap_index: i16,
    sol_text: String, // 1024 chars
    eol_text: String, // 1024 chars
    vertices: Vec<Vertex>,
    polygons: Vec<Polygon>,
    triggers: Vec<Trigger>,
    motions: Vec<Motion>,
    groups: Vec<Group>,
    marks: Vec<Mark>
}

fn eat_bytes<T: ReadBytesExt>(buf: &mut T, len: usize) {
    for i in 0..len {
        let _ = buf.read_u8();
    }
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

fn read_vertex<T: ReadBytesExt>(mut buf: &mut T) -> Vertex {
    let mut v = Vertex { ..Default::default() };

    v.flags = buf.read_u16::<LittleEndian>().unwrap();
    v.x = buf.read_f32::<LittleEndian>().unwrap();
    v.y = buf.read_f32::<LittleEndian>().unwrap();
    v.z = buf.read_f32::<LittleEndian>().unwrap();
    v.sprite_index = buf.read_i16::<LittleEndian>().unwrap();
    v.noise_index = buf.read_i16::<LittleEndian>().unwrap();
    v.lifeform_index = buf.read_i16::<LittleEndian>().unwrap();
    v.orientation = buf.read_i16::<LittleEndian>().unwrap();
    v.group_index = buf.read_i16::<LittleEndian>().unwrap();
    v.item_index = buf.read_i16::<LittleEndian>().unwrap();
    v.trigger_index = buf.read_i16::<LittleEndian>().unwrap();

    eat_bytes(&mut buf, 4);

    return v;
}

fn read_polygon<T: ReadBytesExt>(mut buf: &mut T) -> Polygon {
    let mut p = Polygon { ..Default::default() };

    p.flags = buf.read_u16::<LittleEndian>().unwrap();
    p.polygon_link = buf.read_i16::<LittleEndian>().unwrap();
    p.texture_index = buf.read_i16::<LittleEndian>().unwrap();
    p.bmp_x[0] = buf.read_f32::<LittleEndian>().unwrap();
    p.bmp_x[1] = buf.read_f32::<LittleEndian>().unwrap();
    p.bmp_y[0] = buf.read_f32::<LittleEndian>().unwrap();
    p.bmp_y[1] = buf.read_f32::<LittleEndian>().unwrap();
    for i in 0..8 {
        p.indices[i] = buf.read_i16::<LittleEndian>().unwrap();
    }
    p.animation_index = buf.read_i16::<LittleEndian>().unwrap();
    p.texture_dx = buf.read_f32::<LittleEndian>().unwrap();
    p.texture_dy = buf.read_f32::<LittleEndian>().unwrap();
    p.motion_index = buf.read_i16::<LittleEndian>().unwrap();
    p.group_index = buf.read_i16::<LittleEndian>().unwrap();
    p.texture_x_offset = buf.read_f32::<LittleEndian>().unwrap();
    p.texture_y_offset = buf.read_f32::<LittleEndian>().unwrap();
    let _ = buf.read_i16::<LittleEndian>().unwrap();
    p.damage_animation_index = buf.read_i16::<LittleEndian>().unwrap();
    let _ = buf.read_i16::<LittleEndian>().unwrap();

    eat_bytes(&mut buf, 62);

    return p;
}

fn read_trigger<T: ReadBytesExt>(mut buf: &mut T) -> Trigger {
    let mut t = Trigger { ..Default::default() };

    t.name = read_string(&mut buf, 32);
    t.flags = buf.read_u16::<LittleEndian>().unwrap();
    t.radius = buf.read_f32::<LittleEndian>().unwrap();
    t.height = buf.read_f32::<LittleEndian>().unwrap();
    t.item_type = buf.read_i16::<LittleEndian>().unwrap();
    t.sound_index = buf.read_i16::<LittleEndian>().unwrap();

    return t;
}

fn read_motion<T: ReadBytesExt>(mut buf: &mut T) -> Motion {
    let mut m = Motion { ..Default::default() };

    m.name = read_string(&mut buf, 32);
    m.flags = buf.read_u16::<LittleEndian>().unwrap();
    m.dx = buf.read_f32::<LittleEndian>().unwrap();
    m.dy = buf.read_f32::<LittleEndian>().unwrap();
    m.dz = buf.read_f32::<LittleEndian>().unwrap();
    m.num_steps = buf.read_i16::<LittleEndian>().unwrap();
    m.return_delay = buf.read_i16::<LittleEndian>().unwrap();
    m.trigger_index = buf.read_i16::<LittleEndian>().unwrap();
    m.sound_index_start = buf.read_i16::<LittleEndian>().unwrap();
    m.sound_index_run = buf.read_i16::<LittleEndian>().unwrap();
    m.sound_index_stop = buf.read_i16::<LittleEndian>().unwrap();

    return m;
}

fn read_group<T: ReadBytesExt>(mut buf: &mut T) -> Group {
    let mut g = Group { ..Default::default() };

    g.name = read_string(&mut buf, 32);

    return g;
}

fn read_mark<T: ReadBytesExt>(mut buf: &mut T) -> Mark {
    let mut m = Mark { ..Default::default() };

    m.name = read_string(&mut buf, 32);
    m.x = buf.read_i32::<LittleEndian>().unwrap();
    m.y = buf.read_i32::<LittleEndian>().unwrap();
    m.z = buf.read_i32::<LittleEndian>().unwrap();
    m.rotation = buf.read_f64::<LittleEndian>().unwrap();
    m.elevation = buf.read_f64::<LittleEndian>().unwrap();

    return m;
}

fn read_level_data<T: ReadBytesExt>(mut buf: &mut T) -> Level {
    let mut level = Level { ..Default::default() };
    let hdr_size = buf.read_i16::<LittleEndian>().unwrap();

    level.name = read_string(&mut buf, 32);
    level.author = read_string(&mut buf, 32);
    level.num_vertices = buf.read_i16::<LittleEndian>().unwrap();
    level.num_polygons = buf.read_i16::<LittleEndian>().unwrap();
    level.num_triggers = buf.read_i16::<LittleEndian>().unwrap();
    level.num_motions = buf.read_i16::<LittleEndian>().unwrap();
    level.num_groups = buf.read_i16::<LittleEndian>().unwrap();
    level.num_marks = buf.read_i16::<LittleEndian>().unwrap();
    level.last_mark = buf.read_i16::<LittleEndian>().unwrap();
    level.view_home_rotation = buf.read_f64::<LittleEndian>().unwrap();
    level.view_home_elevation = buf.read_f64::<LittleEndian>().unwrap();
    level.view_home_x = buf.read_i32::<LittleEndian>().unwrap();
    level.view_home_y = buf.read_i32::<LittleEndian>().unwrap();
    level.view_home_z = buf.read_i32::<LittleEndian>().unwrap();
    level.view_rotation = buf.read_f64::<LittleEndian>().unwrap();
    level.view_elevation = buf.read_f64::<LittleEndian>().unwrap();
    level.view_x = buf.read_i32::<LittleEndian>().unwrap();
    level.view_y = buf.read_i32::<LittleEndian>().unwrap();
    level.view_z = buf.read_i32::<LittleEndian>().unwrap();
    level.view_delta = buf.read_i32::<LittleEndian>().unwrap();
    level.view_depth = buf.read_i32::<LittleEndian>().unwrap();
    level.view_width = buf.read_i32::<LittleEndian>().unwrap();
    level.view_height = buf.read_i32::<LittleEndian>().unwrap();
    level.grid_x = buf.read_i32::<LittleEndian>().unwrap();
    level.grid_y = buf.read_i32::<LittleEndian>().unwrap();
    level.grid_z = buf.read_i32::<LittleEndian>().unwrap();
    level.grid_rotation = buf.read_f64::<LittleEndian>().unwrap();
    level.grid_elevation = buf.read_f64::<LittleEndian>().unwrap();
    level.grid_delta = buf.read_i32::<LittleEndian>().unwrap();
    level.grid_spacing = buf.read_i32::<LittleEndian>().unwrap();
    level.grid_size = buf.read_i32::<LittleEndian>().unwrap();
    level.maintain_grid_dist = buf.read_u16::<LittleEndian>().unwrap();
    level.lock_view_to_grid = buf.read_u16::<LittleEndian>().unwrap();
    level.snap_to_grid = buf.read_u16::<LittleEndian>().unwrap();
    level.flags = buf.read_u16::<LittleEndian>().unwrap();
    level.bkg_bitmap_index = buf.read_i16::<LittleEndian>().unwrap();
    level.sol_bitmap_index = buf.read_i16::<LittleEndian>().unwrap();
    level.eol_bitmap_index = buf.read_i16::<LittleEndian>().unwrap();
    level.sol_text = read_string(&mut buf, 1024);
    level.eol_text = read_string(&mut buf, 1024);

    for i in 0..level.num_vertices {
        let v: Vertex = read_vertex(&mut buf);
        level.vertices.push(v);
    }

    for i in 0..level.num_polygons {
        let p: Polygon = read_polygon(&mut buf);
        level.polygons.push(p);
    }

    for i in 0..level.num_triggers {
        let t: Trigger = read_trigger(&mut buf);
        level.triggers.push(t);
    }

    for i in 0..level.num_motions {
        let m: Motion = read_motion(&mut buf);
        level.motions.push(m);
    }

    for i in 0..level.num_groups {
        let g: Group = read_group(&mut buf);
        level.groups.push(g);
    }

    for i in 0..level.num_marks {
        let m: Mark = read_mark(&mut buf);
        level.marks.push(m);
    }

    return level
}


fn main() {
//    fs::create_dir("./out").unwrap();

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
                let l = read_level_data(&mut buffer);
                println!("read level {} by {}", l.name, l.author);
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
        let mut img = Image::new(bmp.width * 2, bmp.height * 2);
//        let mut img = Image::new(bmp.width, bmp.height);

        let mut to_scale = vec![0u32; (bmp.width * bmp.height) as usize];
        let mut scaled = vec![0u32; (bmp.width * bmp.height * 4) as usize];

        for y in 0..bmp.height {
            for x in 0..bmp.width {
                let y_flip: u32 = (y as i32 - (bmp.height - 1) as i32).abs() as u32;
                let pixel = bmp.data[((y_flip * bmp.width) + x) as usize];
                let r = palettes[t.colour_idx].r[pixel as usize];
                let g = palettes[t.colour_idx].g[pixel as usize];
                let b = palettes[t.colour_idx].b[pixel as usize];

                let p: u32 = (r as u32) | ((g as u32) << 8) | ((b as u32) << 16) | ((255 as u32) << 24);
                to_scale[((y * bmp.width) + x) as usize] = p;
//                img.set_pixel(x, y, Pixel { r: r, g: g, b: b });
            }
        }

        superxbr::scale(to_scale.as_mut_slice(), scaled.as_mut_slice(), bmp.width as i32, bmp.height as i32);

//        for (x, y) in img.coordinates() {
        for y in 0..bmp.height*2 {
            for x in 0..bmp.width*2 {
                let idx: usize = ((y * bmp.width*2) + x) as usize;
                let r: u8 = (scaled[idx] & 0xff) as u8;
                let g: u8 = ((scaled[idx] >> 8) & 0xff) as u8;
                let b: u8 = ((scaled[idx] >> 16) & 0xff) as u8;

                //            println!("r {} g {} b {}", r, g, b);
                img.set_pixel(x, y, Pixel { r: r, g: g, b: b });
            }
        }

        println!("Saving texture {}...", i);
        match img.save(&format!("out/{}.bmp", i)[..]) {
            Err(err) => panic!("{}", err),
            Ok(x) => x
        };
    }
}
