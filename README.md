# SpaceTravel — Snoopy Solar System (Software Renderer)

[![Watch video (YouTube)](https://img.youtube.com/vi/ry4oAwbALA8/hqdefault.jpg)](https://youtu.be/ry4oAwbALA8)

A single executable that merges the Bianca-style solar tour with the Snoopy spaceship. Everything you see (planets, rings, Snoopy) is rasterized on the CPU with our custom framebuffer + Raylib window. Controls, warp keys, and camera behavior match BiancaCalderon/SpaceTravel so grading can be done side-by-side.

## Run it

```powershell
cargo run --release
```

Release mode is recommended for a stable 60–90 FPS on mid-tier laptops. Debug builds also work but with lower frame rates.

## Controls (same as Bianca’s spec)

- `W` / `S`: fly the Snoopy ship forward and backward
- `A` / `D`: strafe left / right (ship-relative)
- `Q` / `E`: subir / bajar (ascend / descend)
- Arrow keys: rotate camera around the ship
- `Z` / `X`: zoom (changes world→screen pixel scale)
- `1`: warp near the sun
- `2`..`8`: warp near Asteroide, Rocoso, Tierra, Cristal, Fuego, Agua y Nube (same ordering as Bianca’s README)
- `B`: toggle bird-eye camera (zenith)
- `P`: toggle high-quality spheres (more stacks/slices + full shader) vs. performance mode
- `Esc`: exit

## What you’ll see

- Central star plus 7 stylized planets with Static-Shader patterns: rocky, gas, sci-fi, lava, ice, etc.
- Two procedural rings on Cristal, optional halo on Agua.
- Two orbiting moons (Rocoso + Cristal) with their own shading.
- Snoopy mesh rendered from `models/Snoopy.obj` using the base color read from `models/Snoopy.mtl`.
- Warp titles update to show which body you’re locked on (`Snoopy Solar System — Tierra`, etc.).
- Orbit guides for planets + moons (drawn last so they stay visible).
- Bird-eye view for grading camera comparisons.

## Performance + Shading Notes

- `P` (performance switch) drops sphere resolution and simplifies shader paths when FPS matters.
- Uniforms map to the same Static/Dynamic shader modes used in prior branches, so color palettes match past deliveries.
- Everything is CPU-rendered with barycentric triangles, depth buffer, and procedural fragment shader.

## Grading checklist

- ✅ One binary (`cargo run --release`) that covers the solar system + Snoopy ship; no secondary target needed.
- ✅ Controls and warp keys identical to BiancaCalderon/SpaceTravel for 1:1 evaluation.
- ✅ Snoopy mesh + material colors loaded from `/models`, matching repository assets.
- ✅ All Rust warnings addressed; build is clean on stable 1.78+.
- ✅ README documents how to run, what to test, and where each rubric item lives.

## Repo layout

- `src/main.rs` — entry point with camera, controls, warp logic, Snoopy drawing, and raster loop.
- `src/framebuffer.rs`, `src/triangle.rs`, `src/shader.rs` — software renderer core.
- `src/obj_loader.rs` — OBJ triangulation (used for Snoopy); automatically centers meshes.
- `models/` — includes `Snoopy.obj` + `.mtl`.

Any other experimental binaries (old wireframe viewer, solar prototype) were consolidated here so there’s nothing extra to grade.
