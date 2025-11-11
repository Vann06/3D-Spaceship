use glam::Vec3; // Removed unused Vec4 after planet cleanup

#[derive(Clone)]
pub struct Mesh {
    pub positions: Vec<Vec3>,
    pub indices: Vec<[u32;3]>,
}
impl Mesh {
    pub fn new(positions: Vec<Vec3>, indices: Vec<[u32;3]>) -> Self { Self { positions, indices } }
}
