macro_rules! red {
    ($x:expr) => ((x >> 0) & 0xff)
}

macro_rules! gree {
    ($x:expr) => ((x >> 8) & 0xff)
}

macro_rules! blue {
    ($x:expr) => ((x >> 16) & 0xff)
}

macro_rules! alpha {
    ($x:expr) => ((x >> 24) & 0xff)
}


pub fn minv(v: Vec<f64>) -> f64 {
    v.iter().cloned().fold(0./0., f64::min)
}

pub fn maxv(v: Vec<f64>) -> f64 {
    v.iter().cloned().fold(0./0., f64::max)
}

pub fn scale(data: &[u32], out: &mut [u32], width: u32, height: u32) {
    let f: u32 = 2;

    let outw = width*f;
    let outh = height*f;

    let ws: [f32; 6] = [2.0, 1.0, -1.0, 4.0, -1.0, 1.0];

    for y in 0..outh {
        for x in 0..outw {
            let cx = x/f;
            let cy = y/f;

            for sx in 0..4 {
                for sy in 0..4 {
//                        let csy = clamp(sy as i32 - 1 + cy as i32, 0, height - 1);
//                        let csx = clamp(sx as i32 - 1 + cx as i32, 0, width - 1);

//                        let sample = data[(csy*width + csx) as usize];
//                        r = red!(sample) as f32;
                }
            }
        }
    }
}