use glam::{Vec3, Vec3Swizzles};

#[derive(Copy, Clone, Default)]
pub struct Fragment {
    pub world_pos: Vec3, // posición 3D en mundo (interpolada)
}

#[derive(Copy, Clone, Default)]
pub struct Uniforms {
    pub time: f32,
}

#[derive(Copy, Clone, Debug)]
pub enum Mode { Rocky = 0, Gas = 1, Scifi = 2 }

#[derive(Copy, Clone, Default)]
pub struct ShaderUniforms {
    pub time: f32,
    pub mode: u32,
    pub light_dir: Vec3,
    pub camera_pos: Vec3,
    pub ring_inner: f32,
    pub ring_outer: f32,
    pub ring_enabled: bool,
}

// ---------- utilidades ----------
#[inline] fn clamp01(x: f32) -> f32 { x.max(0.0).min(1.0) }
#[inline] fn vec3_clamp01(v: Vec3) -> Vec3 { Vec3::new(clamp01(v.x), clamp01(v.y), clamp01(v.z)) }

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
fn fbm(mut x: f32, mut y: f32, oct: i32) -> f32 {
    let mut amp = 0.5;
    let mut f = 0.0;
    for _ in 0..oct { f += amp * noise(x, y); x *= 2.02; y *= 2.02; amp *= 0.5; }
    f
}
fn lighting(n: Vec3, view: Vec3) -> (f32,f32) {
    let l = Vec3::new(0.6,0.7,0.2).normalize();
    let lam = n.dot(l).max(0.0);
    let rim = (1.0 - n.dot(view).max(0.0)).powf(2.0);
    (lam, rim)
}
fn lon_lat_from_pos(p: Vec3) -> (f32, f32, Vec3) {
    let n = p.normalize();
    (n.z.atan2(n.x), n.y.asin(), n)
}

// ---------- planeta rocoso ----------
pub fn rocky_planet(fragment: Fragment, u: ShaderUniforms) -> Vec3 {
    let (lon, lat, n) = lon_lat_from_pos(fragment.world_pos);
    let view = (u.camera_pos - fragment.world_pos).normalize();
    let (lam, rim) = lighting(n, view);

    // Procedural layers for terrain
    let cont = fbm(lon*2.5, lat*2.5, 5);
    let land_mask = ((cont - 0.50) * 20.0).tanh() * 0.5 + 0.5;
    let equator = 1.0 - ((lat.abs() - 0.2) / 0.4).clamp(0.0, 1.0);
    let ice = ((lat.abs() - 0.85) / 0.10).clamp(0.0, 1.0);
    let rough = fbm(lon*8.0 + 0.3, lat*8.0 + 0.1*u.time, 5);
    let strata = (lat*40.0 + lon*20.0).sin() * 0.5 + 0.5;
    let mineral = fbm(lon*16.0 + 0.5*u.time, lat*16.0, 3);

    let ocean = Vec3::new(0.05,0.12,0.30);
    let land  = Vec3::new(0.24,0.32,0.12);
    let desert= Vec3::new(0.75,0.62,0.35);
    let icec  = Vec3::new(0.90,0.95,1.00);
    let strata_col = Vec3::new(0.35,0.28,0.18).lerp(Vec3::new(0.55,0.45,0.30), strata);
    let mineral_col = Vec3::new(0.7,0.6,0.8).lerp(Vec3::new(0.4,0.3,0.5), mineral);

    let base_land = land*(1.0-0.7*equator) + desert*(0.7*equator);
    let mut base = ocean*(1.0-land_mask) + base_land*land_mask;
    base = base*(1.0-ice) + icec*ice;
    base = base.lerp(strata_col, 0.15*strata);
    base = base.lerp(mineral_col, 0.10*mineral);
    base *= 0.9 + 0.2*rough;

    // Subtle emission for realism
    let emission = 0.08 * (strata * mineral * land_mask);
    let mut col = base * (0.22 + 0.78*lam) + Vec3::splat(1.0) * (0.13*rim) + Vec3::splat(emission);
    // Si es luna, añade brillo blanco (emisión)
    if u.mode == 99 {
        col += Vec3::splat(0.7); // brillo blanco extra
    }
    vec3_clamp01(col)
}

// ---------- gigante gaseoso ----------
pub fn gas_giant(fragment: Fragment, u: ShaderUniforms) -> Vec3 {
    let (lon, lat, n) = lon_lat_from_pos(fragment.world_pos);
    let view = (u.camera_pos - fragment.world_pos).normalize();
    let (lam, rim) = lighting(n, view);

    let warp = fbm(lon*2.0 + u.time*0.2, lat*4.0, 4);
    let band = (lat*18.0 + 0.15*warp).sin();
    let k = 0.5 + 0.5*band;
    let band_a = Vec3::new(0.85,0.60,0.40);
    let band_b = Vec3::new(0.95,0.85,0.60);
    let mut base = band_a.lerp(band_b, k);

    // eye of storm
    let lat0 = 0.2;
    let lon0 = -0.7 + 0.15*(0.3*u.time).sin();
    let d = ((lat - lat0).powf(2.0) + (lon - lon0).powf(2.0)).sqrt();
    let eye = (1.0 - (d / 0.25).clamp(0.0,1.0)).powf(2.0);
    base = base.lerp(Vec3::new(1.0,0.95,0.85), eye);

    // Si es anillo, ilumina y cambia color
    if u.ring_enabled {
        let xz = fragment.world_pos.xz();
        let r = xz.length();
        let mask = (r >= u.ring_inner) && (r <= u.ring_outer);
        if !mask { return Vec3::new(0.0, 0.0, 0.0); }

        // Bandas sinusoidales y granulado procedural
        let bands = 0.5 + 0.5 * (r * 120.0 + u.time * 0.5).sin();
        let grains = fbm(r * 25.0, 0.0, 3);
        let colorA = Vec3::new(0.95, 0.92, 0.85); // blanco-amarillo
        let colorB = Vec3::new(0.7, 0.7, 0.8);    // azul-gris
        let colorC = Vec3::new(0.8, 0.6, 1.0);    // violeta claro
        let base = colorA.lerp(colorB, 0.6 * bands + 0.4 * grains).lerp(colorC, 0.2 * bands);

        // Brillo en el borde (rim)
        let rim = (1.0 - ((r - u.ring_inner) / (u.ring_outer - u.ring_inner)).abs()).powf(2.0);
        let light = 0.7 + 0.3 * rim;

        // Opacidad variable (más transparente en los bordes)
        // Si tu framebuffer soporta alpha, puedes usar: let alpha = 0.85 * rim;
        // Aquí solo se suma el color, pero puedes adaptar para blending si lo implementas

        return vec3_clamp01(base * light + Vec3::splat(0.18 * rim));
    }

    let col = base * (0.25 + 0.75*lam) + Vec3::splat(1.0)*(0.08*rim);
    vec3_clamp01(col)
}

// ---------- planeta sci-fi (lava / ciudad) ----------
pub fn scifi_planet(fragment: Fragment, u: ShaderUniforms) -> Vec3 {
    let (lon, lat, n) = lon_lat_from_pos(fragment.world_pos);
    let view = (u.camera_pos - fragment.world_pos).normalize();
    let (lam, rim) = lighting(n, view);

    let f = fbm(lon*7.0 + 0.2*u.time, lat*7.0, 5);
    let cracks = ((0.62 - f) * 50.0).clamp(0.0,1.0);

    // Previous look: dark, subtle emission, less purple
    let core = Vec3::new(0.18, 0.08, 0.22); // dark purple
    let lava = Vec3::new(1.0, 0.5, 0.2); // orange lava
    let base = core.lerp(lava, cracks * 0.7 + rim * 0.2);

    // Emission in cracks and rim
    let emission = cracks * (0.5 + 0.5*(1.0 - lam)) + rim * 0.15;
    let col = base * (0.25 + 0.75*lam) + Vec3::new(1.0,0.5,0.2)*emission;
    vec3_clamp01(col)
}

// ring mask helper: returns true if fragment should be visible for ring
fn ring_mask(fragment: Fragment, u: ShaderUniforms) -> bool {
    if !u.ring_enabled { return true; }
    let xz = fragment.world_pos.xz();
    let r = xz.length();
    (r >= u.ring_inner) && (r <= u.ring_outer)
}

pub fn shade_fragment(fragment: Fragment, u: ShaderUniforms) -> Vec3 {
    // if ring mode active, mask by radius
    if u.ring_enabled && !ring_mask(fragment, u) {
        return Vec3::new(0.0, 0.0, 0.0); // background color (black)
    }

    match u.mode {
        0 => rocky_planet(fragment, u),
        1 => gas_giant(fragment, u),
        2 => scifi_planet(fragment, u),
        _ => rocky_planet(fragment, u),
    }
}
