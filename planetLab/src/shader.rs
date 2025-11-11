use glam::Vec3;

#[derive(Copy, Clone, Default)]
pub struct Fragment {
    pub world_pos: Vec3,
}

#[derive(Copy, Clone, Default)]
pub struct StarUniforms {
    pub time: f32,
    pub camera_pos: Vec3,
    pub temp_kelvin: f32,
    pub disp_amp: f32,
    pub disp_freq: f32,
    pub speed: f32,
    pub noise_mode: u32, // 1=Perlin, 2=Simplex, 3=Cellular
}

#[inline]
fn clamp01(x: f32) -> f32 { x.max(0.0).min(1.0) }
#[inline]
fn vec3_clamp01(v: Vec3) -> Vec3 { Vec3::new(clamp01(v.x), clamp01(v.y), clamp01(v.z)) }

fn hash12(x: f32, y: f32) -> f32 {
    let mut p3x = (x * 0.1031).fract();
    let mut p3y = (y * 0.1031).fract();
    let mut p3z = (x * 0.1031).fract();
    let d = p3x * (p3y + 33.33) + p3y * (p3z + 33.33) + p3z * (p3x + 33.33);
    p3x = (p3x + d).fract(); p3y = (p3y + d).fract(); p3z = (p3z + d).fract();
    ((p3x + p3y) * p3z).fract()
}

fn noise(x: f32, y: f32) -> f32 {
    let ix = x.floor(); let iy = y.floor();
    let fx = x - ix;    let fy = y - iy;
    let a = hash12(ix, iy);
    let b = hash12(ix + 1.0, iy);
    let c = hash12(ix, iy + 1.0);
    let d = hash12(ix + 1.0, iy + 1.0);
    let ux = fx * fx * (3.0 - 2.0 * fx);
    let uy = fy * fy * (3.0 - 2.0 * fy);
    let m1 = a + (b - a) * ux;
    let m2 = c + (d - c) * ux;
    m1 + (m2 - m1) * uy
}

// Perlin-like 2D using hashed gradient directions
fn perlin2d(x: f32, y: f32) -> f32 {
    let x0 = x.floor(); let y0 = y.floor();
    let xf = x - x0;    let yf = y - y0;
    let s = |u: f32| u*u*(3.0-2.0*u);

    let grad = |ix: f32, iy: f32, dx: f32, dy: f32| {
        let a = hash12(ix, iy) * std::f32::consts::TAU;
        let gx = a.cos(); let gy = a.sin();
        gx*dx + gy*dy
    };

    let n00 = grad(x0,   y0,   xf,     yf);
    let n10 = grad(x0+1.0,y0,  xf-1.0, yf);
    let n01 = grad(x0,   y0+1.0, xf,   yf-1.0);
    let n11 = grad(x0+1.0,y0+1.0, xf-1.0, yf-1.0);

    let u = s(xf); let v = s(yf);
    let nx0 = n00 + (n10 - n00) * u;
    let nx1 = n01 + (n11 - n01) * u;
    let n = nx0 + (nx1 - nx0) * v;
    0.5 * (n + 1.0)
}

// Simplex noise 2D (public-domain style minimal impl)
fn simplex2d(xin: f32, yin: f32) -> f32 {
    // Skewing/Unskewing factors
    const F2: f32 = 0.3660254037844386; // (sqrt(3)-1)/2
    const G2: f32 = 0.21132486540518713; // (3-sqrt(3))/6

    let s = (xin + yin) * F2;
    let i = (xin + s).floor();
    let j = (yin + s).floor();
    let t = (i + j) * G2;
    let x0 = xin - (i - t);
    let y0 = yin - (j - t);

    let (i1, j1) = if x0 > y0 { (1.0, 0.0) } else { (0.0, 1.0) };

    let x1 = x0 - i1 + G2;
    let y1 = y0 - j1 + G2;
    let x2 = x0 - 1.0 + 2.0*G2;
    let y2 = y0 - 1.0 + 2.0*G2;

    let ii = i; let jj = j;
    let corner = |dx: f32, dy: f32, ox: f32, oy: f32| {
        let t = 0.5 - dx*dx - dy*dy;
        if t < 0.0 { 0.0 } else {
            let t2 = t*t;
            let a = hash12(ox, oy) * std::f32::consts::TAU;
            let gx = a.cos(); let gy = a.sin();
            (t2*t2) * (gx*dx + gy*dy)
        }
    };

    let n0 = corner(x0, y0, ii,     jj);
    let n1 = corner(x1, y1, ii+i1,  jj+j1);
    let n2 = corner(x2, y2, ii+1.0, jj+1.0);
    // Scale to 0..1 roughly
    (n0 + n1 + n2) * 70.0 * 0.5 + 0.5
}

fn fbm(mut x: f32, mut y: f32, oct: i32) -> f32 {
    let mut amp = 0.5;
    let mut f = 0.0;
    for _ in 0..oct { f += amp * noise(x, y); x *= 2.02; y *= 2.02; amp *= 0.5; }
    f
}

fn cellular(x: f32, y: f32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let mut dmin = 1e9f32;
    for dy in -1..=1 {
        for dx in -1..=1 {
            let fx = (xi + dx) as f32;
            let fy = (yi + dy) as f32;
            let hx = hash12(fx, fy);
            let hy = hash12(fx + 17.0, fy + 31.0);
            let px = fx + hx;
            let py = fy + hy;
            let ddx = x - px;
            let ddy = y - py;
            let d = (ddx*ddx + ddy*ddy).sqrt();
            if d < dmin { dmin = d; }
        }
    }
    1.0 - (dmin / 1.414).clamp(0.0,1.0)
}

fn kelvin_to_rgb(temp_k: f32) -> Vec3 {
    let t = (temp_k / 6500.0).clamp(0.2, 2.0);
    let c1 = Vec3::new(0.9, 0.2, 0.0);
    let c2 = Vec3::new(1.0, 0.5, 0.0);
    let c3 = Vec3::new(1.0, 0.9, 0.2);
    let c4 = Vec3::new(0.95, 0.98, 1.0);
    if t < 0.6 { c1.lerp(c2, (t-0.2)/0.4) }
    else if t < 1.0 { c2.lerp(c3, (t-0.6)/0.4) }
    else { c3.lerp(c4, (t-1.0)/1.0) }
}

fn compute_layers(lon: f32, lat: f32, s: f32, t: f32, mode: u32) -> (f32,f32,f32,f32) {
    // returns (base, ridge, cell, pulse)
    let pulse = (t * 1.2).sin() * 0.5 + 0.5;
    match mode {
        1 => { // Perlin
            let b = {
                let mut x = lon * 3.2 * s + 0.28*t;
                let mut y = lat * 3.2 * s - 0.23*t;
                let mut amp = 0.5; let mut acc = 0.0;
                for _ in 0..5 { acc += amp * perlin2d(x,y); x*=2.02; y*=2.02; amp*=0.5; }
                acc
            };
            let n = perlin2d(lon*2.1*s - 0.2*t, lat*2.1*s + 0.17*t);
            let r = 1.0 - (2.0*n - 1.0).abs();
            let c = perlin2d(lon*4.0*s + 0.1*t, lat*4.0*s - 0.12*t);
            (b, r, c, pulse)
        }
        2 => { // Simplex
            let b = {
                let mut x = lon * 3.0 * s + 0.30*t;
                let mut y = lat * 3.0 * s - 0.25*t;
                let mut amp = 0.5; let mut acc = 0.0;
                for _ in 0..5 { acc += amp * simplex2d(x,y); x*=2.02; y*=2.02; amp*=0.5; }
                acc
            };
            let n = simplex2d(lon*2.0*s - 0.2*t, lat*2.0*s + 0.17*t);
            let r = 1.0 - (2.0*n - 1.0).abs();
            let c = simplex2d(lon*4.0*s + 0.1*t, lat*4.0*s - 0.12*t);
            (b, r, c, pulse)
        }
        _ => { // Cellular + value (default)
            let b = fbm(lon * 3.0 * s + 0.30*t, lat * 3.0 * s - 0.25*t, 5);
            let r = 1.0 - (2.0*noise(lon*2.0*s - 0.2*t, lat*2.0*s + 0.17*t) - 1.0).abs();
            let c = cellular(lon * 4.0 * s + 0.1*t, lat * 4.0 * s - 0.12*t);
            (b, r, c, pulse)
        }
    }
}

pub fn star_color(fragment: Fragment, u: StarUniforms) -> Vec3 {
    let p = fragment.world_pos.normalize();
    let view = (u.camera_pos - fragment.world_pos).normalize();
    let rim = (1.0 - p.dot(view).abs().clamp(0.0,1.0)).powf(3.0);

    let lon = p.z.atan2(p.x);
    let lat = p.y.asin();
    let s = u.disp_freq.max(0.5);

    let t = u.time * u.speed;
    let (base, ridge, cell, pulse) = compute_layers(lon, lat, s, t, u.noise_mode);

    let intensity = (0.55*base + 0.35*ridge + 0.25*cell) * (0.90 + 0.40*pulse);
    let base_col = kelvin_to_rgb(u.temp_kelvin);
    let hot = Vec3::new(1.0, 0.98, 0.95);
    let mut col = base_col.lerp(hot, (intensity*1.2).clamp(0.0,1.0));

    let spikes = (ridge * 1.6 + cell * 1.2) * 0.55;
    let corona = rim * (0.9 + 0.8*pulse) + 0.30*spikes;
    col += Vec3::new(1.0, 0.8, 0.3) * corona;

    vec3_clamp01(col)
}

pub fn star_displacement(normal_ws: Vec3, u: StarUniforms) -> f32 {
    let p = normal_ws;
    let lon = p.z.atan2(p.x);
    let lat = p.y.asin();
    let s = u.disp_freq.max(0.5);
    let t = u.time * u.speed;
    let (base, ridge, cell, _) = compute_layers(lon, lat, s, t, u.noise_mode);
    (0.60*base + 0.35*ridge + 0.30*cell) * u.disp_amp
}
