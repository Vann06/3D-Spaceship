# 3D Spaceship / Snoopy Wireframe Viewer

This project renders a wireframe of the `Snoopy.obj` mesh using a custom software rasterizer (lines + triangle edges) and Raylib for window/display.

## Features
- Manual OBJ parser (`v` + `f` lines, triangulates n-gons and handles negative indices)
- Wireframe rendering via Bresenham line algorithm
- Simple Euler rotation (keys Q/W/X/E/R/T/Y) and translation (arrow keys)
- Adjustable scale (A to shrink, S to grow)
- Basic perspective foreshortening based on vertex Z

## Controls
- Arrow Keys: Move model on screen
- Q / W: Rotate around X axis (- / +)
- E / R: Rotate around Y axis (- / +)
- T / Y: Rotate around Z axis (- / +)
- A / S: Scale down / up
- Close window: Press the window close button or ESC

## Building
Requires Rust toolchain installed.

```
cargo run
```

If Raylib build issues occur on Windows, install required build tools (Visual Studio Build Tools or clang) as Raylib is compiled from source.

## OBJ Loader Notes
- Ignores materials, normals, and texture coordinates for now
- Centers the model around its bounding box center for easier viewing
- Performs triangle fan triangulation for faces with >3 vertices

## Next Steps (Ideas)
- Fill triangles instead of just wireframe (add a basic rasterizer with barycentric coords)
- Implement Z-buffer for proper depth handling
- Support vertex normals and simple Lambert shading
- Texture mapping using `vt` coordinates

## License
Your chosen license goes here.
