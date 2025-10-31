use glam::UVec2;

pub struct Framebuffer {
    pub w: usize,
    pub h: usize,
    pub color: Vec<u32>, // ARGB8
    pub depth: Vec<f32>, // z-buffer
}

impl Framebuffer {
    pub fn new(w: usize, h: usize) -> Self {
        Self { w, h, color: vec![0x00000000; w*h], depth: vec![f32::INFINITY; w*h] }
    }
    #[inline] pub fn clear(&mut self, argb: u32) {
        self.color.fill(argb);
        self.depth.fill(f32::INFINITY);
    }
    #[inline] pub fn idx(&self, x: i32, y: i32) -> Option<usize> {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
            Some(y as usize * self.w + x as usize)
        } else { None }
    }
    #[inline] pub fn put(&mut self, x: i32, y: i32, z: f32, argb: u32) {
        if let Some(i) = self.idx(x, y) {
            if z < self.depth[i] {
                self.depth[i] = z;
                self.color[i] = argb;
            }
        }
    }
    pub fn size(&self) -> UVec2 { UVec2::new(self.w as u32, self.h as u32) }
}
