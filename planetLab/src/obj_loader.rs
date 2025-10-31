use glam::Vec3;
use std::fs;

use crate::mesh::Mesh;

pub fn load_obj(path: &str) -> std::io::Result<Mesh> {
    let txt = fs::read_to_string(path)?;
    let mut positions: Vec<Vec3> = Vec::new();
    let mut indices: Vec<[u32;3]> = Vec::new();

    for line in txt.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }

        if let Some(rest) = line.strip_prefix("v ") {
            let mut it = rest.split_whitespace();
            let x: f32 = it.next().unwrap().parse().unwrap();
            let y: f32 = it.next().unwrap().parse().unwrap();
            let z: f32 = it.next().unwrap().parse().unwrap();
            positions.push(Vec3::new(x,y,z));
        } else if let Some(rest) = line.strip_prefix("f ") {
            // soporta "f a b c" o "f a/... b/... c/..."
            let parts: Vec<&str> = rest.split_whitespace().collect();
            let mut idx = [0u32;3];
            for k in 0..3 {
                let p = parts[k].split('/').next().unwrap();
                idx[k] = p.parse::<u32>().unwrap() - 1;
            }
            indices.push(idx);
        }
    }
    Ok(Mesh::new(positions, indices))
}
