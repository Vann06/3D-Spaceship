use raylib::prelude::*;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vector3>,
    pub faces: Vec<[usize; 3]>, // indices into vertices (0-based)
}

impl Mesh {
    pub fn load_obj<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(&path)?;
        let mut positions: Vec<Vector3> = Vec::new();
        let mut faces: Vec<[usize; 3]> = Vec::new();

        for (lineno, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut parts = line.split_whitespace();
            if let Some(tag) = parts.next() {
                match tag {
                    "v" => {
                        // vertex position
                        let coords: Vec<f32> =
                            parts.filter_map(|p| p.parse::<f32>().ok()).collect();
                        if coords.len() >= 3 {
                            positions.push(Vector3::new(coords[0], coords[1], coords[2]));
                        }
                    }
                    "f" => {
                        // faces can be triangles, quads, or n-gons -> triangulate fan
                        let mut indices: Vec<usize> = Vec::new();
                        for p in parts {
                            if p.is_empty() {
                                continue;
                            }
                            // format variants: v, v/vt, v//vn, v/vt/vn
                            let first = p.split('/').next().unwrap_or("");
                            if first.is_empty() {
                                continue;
                            }
                            if let Ok(idx_raw) = first.parse::<isize>() {
                                // OBJ allows negative indices
                                let idx = if idx_raw < 0 {
                                    // negative indices are relative to end (current size + idx_raw + 1)
                                    (positions.len() as isize + idx_raw) as isize
                                } else {
                                    idx_raw - 1
                                } as isize; // convert to 0-based
                                if idx < 0 || (idx as usize) >= positions.len() {
                                    // skip invalid index
                                    continue;
                                }
                                indices.push(idx as usize);
                            }
                        }
                        if indices.len() >= 3 {
                            for i in 1..(indices.len() - 1) {
                                faces.push([indices[0], indices[i], indices[i + 1]]);
                            }
                        } else if !indices.is_empty() {
                            eprintln!(
                                "Warning: face with <3 vertices at line {} ignored",
                                lineno + 1
                            );
                        }
                    }
                    _ => { /* ignore other tags (vt, vn, usemtl, etc.) */ }
                }
            }
        }

        // Center the model around origin for easier manipulation
        if !positions.is_empty() {
            let mut min = positions[0];
            let mut max = positions[0];
            for v in &positions {
                if v.x < min.x {
                    min.x = v.x;
                }
                if v.y < min.y {
                    min.y = v.y;
                }
                if v.z < min.z {
                    min.z = v.z;
                }
                if v.x > max.x {
                    max.x = v.x;
                }
                if v.y > max.y {
                    max.y = v.y;
                }
                if v.z > max.z {
                    max.z = v.z;
                }
            }
            let center = Vector3::new(
                (min.x + max.x) * 0.5,
                (min.y + max.y) * 0.5,
                (min.z + max.z) * 0.5,
            );
            for v in &mut positions {
                v.x -= center.x;
                v.y -= center.y;
                v.z -= center.z;
            }
        }

        Ok(Mesh {
            vertices: positions,
            faces,
        })
    }
}
