/*

Rust port of Hyllian's original code at http://pastebin.com/cbH8ZQQT.
Original license/copyright notice below.

*******  Super XBR Scaler  *******



Copyright (c) 2016 Hyllian - sergiogdb@gmail.com

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.

*/

macro_rules! red {
    ($x:expr) => (($x >> 0) & 0xff)
}

macro_rules! green {
    ($x:expr) => (($x >> 8) & 0xff)
}

macro_rules! blue {
    ($x:expr) => (($x >> 16) & 0xff)
}

macro_rules! alpha {
    ($x:expr) => (($x >> 24) & 0xff)
}

fn df(a: f32, b: f32) -> f32 {
    (a - b).abs()
}

pub fn minv(v: Vec<f32>) -> f32 {
    v.iter().cloned().fold(0. / 0., f32::min)
}

pub fn maxv(v: Vec<f32>) -> f32 {
    v.iter().cloned().fold(0. / 0., f32::max)
}

pub fn clamp(v: i32, floor: i32, ceil: i32) -> i32 {
    let mut out = v;

    if v < floor {
        out = floor;
    }

    if v > ceil {
        out = ceil;
    }

    return out;
}

pub fn clampf(v: f32, floor: f32, ceil: f32) -> f32 {
    let mut out = v;

    if v < floor {
        out = floor;
    }

    if v > ceil {
        out = ceil;
    }

    return out;
}

pub fn diagonal_edge(mat: [[f32; 4]; 4], wp: [f32; 6]) -> f32 {
    let dw1 = wp[0] * (df(mat[0][2], mat[1][1]) + df(mat[1][1], mat[2][0]) + df(mat[1][3], mat[2][2]) + df(mat[2][2], mat[3][1])) +
        wp[1] * (df(mat[0][3], mat[1][2]) + df(mat[2][1], mat[3][0])) +
        wp[2] * (df(mat[0][3], mat[2][1]) + df(mat[1][2], mat[3][0])) +
        wp[3] * df(mat[1][2], mat[2][1]) +
        wp[4] * (df(mat[0][2], mat[2][0]) + df(mat[1][3], mat[3][1])) +
        wp[5] * (df(mat[0][1], mat[1][0]) + df(mat[2][3], mat[3][2]));

    let dw2 = wp[0] * (df(mat[0][1], mat[1][2]) + df(mat[1][2], mat[2][3]) + df(mat[1][0], mat[2][1]) + df(mat[2][1], mat[3][2])) +
        wp[1] * (df(mat[0][0], mat[1][1]) + df(mat[2][2], mat[3][3])) +
        wp[2] * (df(mat[0][0], mat[2][2]) + df(mat[1][1], mat[3][3])) +
        wp[3] * df(mat[1][1], mat[2][2]) +
        wp[4] * (df(mat[1][0], mat[3][2]) + df(mat[0][1], mat[2][3])) +
        wp[5] * (df(mat[0][2], mat[1][3]) + df(mat[2][0], mat[3][1]));

    (dw1 - dw2)
}

pub fn scale(data: &[u32], out: &mut [u32], width: i32, height: i32) {
    let f: i32 = 2;

    let wgt1: f32 = 0.129633;
    let wgt2: f32 = 0.175068;
    let w1: f32 = -wgt1;
    let w2: f32 = wgt1 + 0.5;
    let w3: f32 = -wgt2;
    let w4: f32 = wgt2 + 0.5;

    let outw = width * f;
    let outh = height * f;

    // Pass 1
    let mut wp: [f32; 6] = [2.0, 1.0, -1.0, 4.0, -1.0, 1.0];

    for y in (0..outh).step_by(2) {
        for x in (0..outw).step_by(2) {
            let cx = x / f;
            let cy = y / f;

            let mut r: [[f32; 4]; 4] = [[0.0; 4]; 4];
            let mut g: [[f32; 4]; 4] = [[0.0; 4]; 4];
            let mut b: [[f32; 4]; 4] = [[0.0; 4]; 4];
            let mut a: [[f32; 4]; 4] = [[0.0; 4]; 4];
            let mut Y: [[f32; 4]; 4] = [[0.0; 4]; 4];

            let x_rng: Vec<i32> = vec![-1, 0, 1, 2];
            let y_rng: Vec<i32> = vec![-1, 0, 1, 2];
            for sx in x_rng.iter() {
                for sy in y_rng.iter() {
                    let csy = clamp(sy + cy as i32, 0, height - 1);
                    let csx = clamp(sx + cx as i32, 0, width - 1);
                    let sample = data[(csy * width + csx) as usize];

                    r[(*sx + 1) as usize][(*sy + 1) as usize] = red!(sample) as f32;
                    g[(*sx + 1) as usize][(*sy + 1) as usize] = green!(sample) as f32;
                    b[(*sx + 1) as usize][(*sy + 1) as usize] = blue!(sample) as f32;
                    a[(*sx + 1) as usize][(*sy + 1) as usize] = alpha!(sample) as f32;

                    Y[(*sx + 1) as usize][(*sy + 1) as usize]
                        = 0.2126 * r[(*sx + 1) as usize][(*sy + 1) as usize]
                        + 0.7152 * g[(*sx + 1) as usize][(*sy + 1) as usize]
                        + 0.0722 * b[(*sx + 1) as usize][(*sy + 1) as usize];
                }
            }

            let min_r: f32 = minv(vec![r[1][1], r[2][1], r[1][2], r[2][2]]);
            let min_g: f32 = minv(vec![g[1][1], g[2][1], g[1][2], g[2][2]]);
			let min_b: f32 = minv(vec![b[1][1], b[2][1], b[1][2], b[2][2]]);
			let min_a: f32 = minv(vec![a[1][1], a[2][1], a[1][2], a[2][2]]);
			let max_r: f32 = maxv(vec![r[1][1], r[2][1], r[1][2], r[2][2]]);
			let max_g: f32 = maxv(vec![g[1][1], g[2][1], g[1][2], g[2][2]]);
			let max_b: f32 = maxv(vec![b[1][1], b[2][1], b[1][2], b[2][2]]);
			let max_a: f32 = maxv(vec![a[1][1], a[2][1], a[1][2], a[2][2]]);

            let d_edge: f32 = diagonal_edge(Y, wp);

            let r1: f32 = w1*(r[0][3] + r[3][0]) + w2*(r[1][2] + r[2][1]);
			let g1: f32 = w1*(g[0][3] + g[3][0]) + w2*(g[1][2] + g[2][1]);
			let b1: f32 = w1*(b[0][3] + b[3][0]) + w2*(b[1][2] + b[2][1]);
			let a1: f32 = w1*(a[0][3] + a[3][0]) + w2*(a[1][2] + a[2][1]);
			let r2: f32 = w1*(r[0][0] + r[3][3]) + w2*(r[1][1] + r[2][2]);
			let g2: f32 = w1*(g[0][0] + g[3][3]) + w2*(g[1][1] + g[2][2]);
			let b2: f32 = w1*(b[0][0] + b[3][3]) + w2*(b[1][1] + b[2][2]);
			let a2: f32 = w1*(a[0][0] + a[3][3]) + w2*(a[1][1] + a[2][2]);

            let (mut rf, mut gf, mut bf, mut af) = (0.0, 0.0, 0.0, 0.0);
            if d_edge <= 0.0 {
                rf = r1;
                gf = g1;
                bf = b1;
                af = a1;
            } else {
                rf = r2;
                gf = g2;
                bf = b2;
                af = a2;
            }

            rf = clampf(rf, min_r, max_r);
            gf = clampf(gf, min_g, max_g);
            bf = clampf(bf, min_b, max_b);
            af = clampf(af, min_a, max_a);

            let ri: i32 = clamp(rf.ceil() as i32, 0, 255);
            let gi: i32 = clamp(gf.ceil() as i32, 0, 255);
            let bi: i32 = clamp(bf.ceil() as i32, 0, 255);
            let ai: i32 = clamp(af.ceil() as i32, 0, 255);

            out[(y*outw + x) as usize] = data[(cy*width + cx) as usize];
            out[(y*outw + x + 1) as usize] = data[(cy*width + cx) as usize];
            out[((y + 1)*outw + x) as usize] = data[(cy*width + cx) as usize];

			out[((y+1)*outw + x+1) as usize] = ((ai << 24) | (bi << 16) | (gi << 8) | ri) as u32;
        }
    }

    // Pass 2
    wp = [2.0, 0.0, 0.0, 0.0, 0.0, 0.0];

    for y in (0..outh).step_by(2) {
        for x in (0..outw).step_by(2) {
            let cx = x / f;
            let cy = y / f;

            let mut r: [[f32; 4]; 4] = [[0.0; 4]; 4];
            let mut g: [[f32; 4]; 4] = [[0.0; 4]; 4];
            let mut b: [[f32; 4]; 4] = [[0.0; 4]; 4];
            let mut a: [[f32; 4]; 4] = [[0.0; 4]; 4];
            let mut Y: [[f32; 4]; 4] = [[0.0; 4]; 4];

            let x_rng: Vec<i32> = vec![-1, 0, 1, 2];
            let y_rng: Vec<i32> = vec![-1, 0, 1, 2];
            for sx in x_rng.iter() {
                for sy in y_rng.iter() {
                    let csy = clamp(sx - sy + y as i32, 0, f*height - 1);
                    let csx = clamp(sx + sy + x as i32, 0, f*width - 1);
                    let sample = out[(csy * outw + csx) as usize];

                    r[(*sx + 1) as usize][(*sy + 1) as usize] = red!(sample) as f32;
                    g[(*sx + 1) as usize][(*sy + 1) as usize] = green!(sample) as f32;
                    b[(*sx + 1) as usize][(*sy + 1) as usize] = blue!(sample) as f32;
                    a[(*sx + 1) as usize][(*sy + 1) as usize] = alpha!(sample) as f32;

                    Y[(*sx + 1) as usize][(*sy + 1) as usize]
                        = 0.2126 * r[(*sx + 1) as usize][(*sy + 1) as usize]
                        + 0.7152 * g[(*sx + 1) as usize][(*sy + 1) as usize]
                        + 0.0722 * b[(*sx + 1) as usize][(*sy + 1) as usize];
                }
            }

            let min_r: f32 = minv(vec![r[1][1], r[2][1], r[1][2], r[2][2]]);
            let min_g: f32 = minv(vec![g[1][1], g[2][1], g[1][2], g[2][2]]);
			let min_b: f32 = minv(vec![b[1][1], b[2][1], b[1][2], b[2][2]]);
			let min_a: f32 = minv(vec![a[1][1], a[2][1], a[1][2], a[2][2]]);
			let max_r: f32 = maxv(vec![r[1][1], r[2][1], r[1][2], r[2][2]]);
			let max_g: f32 = maxv(vec![g[1][1], g[2][1], g[1][2], g[2][2]]);
			let max_b: f32 = maxv(vec![b[1][1], b[2][1], b[1][2], b[2][2]]);
			let max_a: f32 = maxv(vec![a[1][1], a[2][1], a[1][2], a[2][2]]);

            let mut d_edge: f32 = diagonal_edge(Y, wp);

            let mut r1: f32 = w3*(r[0][3] + r[3][0]) + w4*(r[1][2] + r[2][1]);
			let mut g1: f32 = w3*(g[0][3] + g[3][0]) + w4*(g[1][2] + g[2][1]);
			let mut b1: f32 = w3*(b[0][3] + b[3][0]) + w4*(b[1][2] + b[2][1]);
			let mut a1: f32 = w3*(a[0][3] + a[3][0]) + w4*(a[1][2] + a[2][1]);
			let mut r2: f32 = w3*(r[0][0] + r[3][3]) + w4*(r[1][1] + r[2][2]);
			let mut g2: f32 = w3*(g[0][0] + g[3][3]) + w4*(g[1][1] + g[2][2]);
			let mut b2: f32 = w3*(b[0][0] + b[3][3]) + w4*(b[1][1] + b[2][2]);
			let mut a2: f32 = w3*(a[0][0] + a[3][3]) + w4*(a[1][1] + a[2][2]);

            let (mut rf, mut gf, mut bf, mut af) = (0.0, 0.0, 0.0, 0.0);
            if d_edge <= 0.0 {
                rf = r1;
                gf = g1;
                bf = b1;
                af = a1;
            } else {
                rf = r2;
                gf = g2;
                bf = b2;
                af = a2;
            }

            rf = clampf(rf, min_r, max_r);
            gf = clampf(gf, min_g, max_g);
            bf = clampf(bf, min_b, max_b);
            af = clampf(af, min_a, max_a);

            let mut ri: i32 = clamp(rf.ceil() as i32, 0, 255);
            let mut gi: i32 = clamp(gf.ceil() as i32, 0, 255);
            let mut bi: i32 = clamp(bf.ceil() as i32, 0, 255);
            let mut ai: i32 = clamp(af.ceil() as i32, 0, 255);

			out[(y*outw + x+1) as usize] = ((ai << 24) | (bi << 16) | (gi << 8) | ri) as u32;

            for sx in x_rng.iter() {
                for sy in y_rng.iter() {
                    let csy = clamp(sx - sy + 1 + y as i32, 0, f*height - 1);
                    let csx = clamp(sx + sy - 1 + x as i32, 0, f*width - 1);
                    let sample = out[(csy * outw + csx) as usize];

                    r[(*sx + 1) as usize][(*sy + 1) as usize] = red!(sample) as f32;
                    g[(*sx + 1) as usize][(*sy + 1) as usize] = green!(sample) as f32;
                    b[(*sx + 1) as usize][(*sy + 1) as usize] = blue!(sample) as f32;
                    a[(*sx + 1) as usize][(*sy + 1) as usize] = alpha!(sample) as f32;

                    Y[(*sx + 1) as usize][(*sy + 1) as usize]
                        = 0.2126 * r[(*sx + 1) as usize][(*sy + 1) as usize]
                        + 0.7152 * g[(*sx + 1) as usize][(*sy + 1) as usize]
                        + 0.0722 * b[(*sx + 1) as usize][(*sy + 1) as usize];
                }
            }

            d_edge = diagonal_edge(Y, wp);

            r1 = w3*(r[0][3] + r[3][0]) + w4*(r[1][2] + r[2][1]);
			g1 = w3*(g[0][3] + g[3][0]) + w4*(g[1][2] + g[2][1]);
			b1 = w3*(b[0][3] + b[3][0]) + w4*(b[1][2] + b[2][1]);
			a1 = w3*(a[0][3] + a[3][0]) + w4*(a[1][2] + a[2][1]);
			r2 = w3*(r[0][0] + r[3][3]) + w4*(r[1][1] + r[2][2]);
			g2 = w3*(g[0][0] + g[3][3]) + w4*(g[1][1] + g[2][2]);
			b2 = w3*(b[0][0] + b[3][3]) + w4*(b[1][1] + b[2][2]);
			a2 = w3*(a[0][0] + a[3][3]) + w4*(a[1][1] + a[2][2]);

            let (mut rf, mut gf, mut bf, mut af) = (0.0, 0.0, 0.0, 0.0);
            if d_edge <= 0.0 {
                rf = r1;
                gf = g1;
                bf = b1;
                af = a1;
            } else {
                rf = r2;
                gf = g2;
                bf = b2;
                af = a2;
            }

            rf = clampf(rf, min_r, max_r);
            gf = clampf(gf, min_g, max_g);
            bf = clampf(bf, min_b, max_b);
            af = clampf(af, min_a, max_a);

            ri = clamp(rf.ceil() as i32, 0, 255);
            gi = clamp(gf.ceil() as i32, 0, 255);
            bi = clamp(bf.ceil() as i32, 0, 255);
            ai = clamp(af.ceil() as i32, 0, 255);

			out[((y+1)*outw + x) as usize] = ((ai << 24) | (bi << 16) | (gi << 8) | ri) as u32;
        }
    }

    // Pass 3
    wp = [2.0, 1.0, -1.0, 40.0, -1.0, 1.0];

    for y in (0..outh).rev() {
        for x in (0..outw).rev() {
            let cx = x / f;
            let cy = y / f;

            let mut r: [[f32; 4]; 4] = [[0.0; 4]; 4];
            let mut g: [[f32; 4]; 4] = [[0.0; 4]; 4];
            let mut b: [[f32; 4]; 4] = [[0.0; 4]; 4];
            let mut a: [[f32; 4]; 4] = [[0.0; 4]; 4];
            let mut Y: [[f32; 4]; 4] = [[0.0; 4]; 4];

            let x_rng: Vec<i32> = vec![-2, -1, 0, 1];
            let y_rng: Vec<i32> = vec![-2, -1, 0, 1];
            for sx in x_rng.iter() {
                for sy in y_rng.iter() {
                    let csy = clamp(sy + y as i32, 0, f * height - 1);
                    let csx = clamp(sx + x as i32, 0, f * width - 1);
                    let sample = out[(csy * outw + csx) as usize];

                    r[(*sx + 2) as usize][(*sy + 2) as usize] = red!(sample) as f32;
                    g[(*sx + 2) as usize][(*sy + 2) as usize] = green!(sample) as f32;
                    b[(*sx + 2) as usize][(*sy + 2) as usize] = blue!(sample) as f32;
                    a[(*sx + 2) as usize][(*sy + 2) as usize] = alpha!(sample) as f32;

                    Y[(*sx + 2) as usize][(*sy + 2) as usize]
                        = 0.2126 * r[(*sx + 2) as usize][(*sy + 2) as usize]
                        + 0.7152 * g[(*sx + 2) as usize][(*sy + 2) as usize]
                        + 0.0722 * b[(*sx + 2) as usize][(*sy + 2) as usize];
                }
            }

            let min_r: f32 = minv(vec![r[1][1], r[2][1], r[1][2], r[2][2]]);
            let min_g: f32 = minv(vec![g[1][1], g[2][1], g[1][2], g[2][2]]);
            let min_b: f32 = minv(vec![b[1][1], b[2][1], b[1][2], b[2][2]]);
            let min_a: f32 = minv(vec![a[1][1], a[2][1], a[1][2], a[2][2]]);
            let max_r: f32 = maxv(vec![r[1][1], r[2][1], r[1][2], r[2][2]]);
            let max_g: f32 = maxv(vec![g[1][1], g[2][1], g[1][2], g[2][2]]);
            let max_b: f32 = maxv(vec![b[1][1], b[2][1], b[1][2], b[2][2]]);
            let max_a: f32 = maxv(vec![a[1][1], a[2][1], a[1][2], a[2][2]]);

            let d_edge: f32 = diagonal_edge(Y, wp);

            let r1: f32 = w1 * (r[0][3] + r[3][0]) + w2 * (r[1][2] + r[2][1]);
            let g1: f32 = w1 * (g[0][3] + g[3][0]) + w2 * (g[1][2] + g[2][1]);
            let b1: f32 = w1 * (b[0][3] + b[3][0]) + w2 * (b[1][2] + b[2][1]);
            let a1: f32 = w1 * (a[0][3] + a[3][0]) + w2 * (a[1][2] + a[2][1]);
            let r2: f32 = w1 * (r[0][0] + r[3][3]) + w2 * (r[1][1] + r[2][2]);
            let g2: f32 = w1 * (g[0][0] + g[3][3]) + w2 * (g[1][1] + g[2][2]);
            let b2: f32 = w1 * (b[0][0] + b[3][3]) + w2 * (b[1][1] + b[2][2]);
            let a2: f32 = w1 * (a[0][0] + a[3][3]) + w2 * (a[1][1] + a[2][2]);

            let (mut rf, mut gf, mut bf, mut af) = (0.0, 0.0, 0.0, 0.0);
            if d_edge <= 0.0 {
                rf = r1;
                gf = g1;
                bf = b1;
                af = a1;
            } else {
                rf = r2;
                gf = g2;
                bf = b2;
                af = a2;
            }

            rf = clampf(rf, min_r, max_r);
            gf = clampf(gf, min_g, max_g);
            bf = clampf(bf, min_b, max_b);
            af = clampf(af, min_a, max_a);

            let mut ri: i32 = clamp(rf.ceil() as i32, 0, 255);
            let mut gi: i32 = clamp(gf.ceil() as i32, 0, 255);
            let mut bi: i32 = clamp(bf.ceil() as i32, 0, 255);
            let mut ai: i32 = clamp(af.ceil() as i32, 0, 255);

            out[(y * outw + x) as usize] = ((ai << 24) | (bi << 16) | (gi << 8) | ri) as u32;
        }
    }
}