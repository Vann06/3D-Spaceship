# StarLab — Estrella Procedural (Rust CPU Rasterizer)

Este repositorio ahora es una build solo de estrella. La estrella/“sol” se genera sobre una sola malla de esfera sin texturas ni materiales precargados. Todo corre en CPU con un rasterizador propio (Rust + minifb + glam): transformación, proyección, z‑buffer y raster de triángulos.

Demostración (GIF): agrega `planetLab/docs/star.gif` para que el README la muestre en Capturas.

Características clave
- Animación continua basada en time y funciones de ruido (fbm, ridge, cellular/Worley‑like).
- Emisión variable y corona con efecto rim^3 y picos energéticos.
- Desplazamiento radial de vértices (flare) por combinación de ruidos.
- Color controlado por temperatura con kelvin_to_rgb (rojo→blanco azulado).

Uniforms
| Nombre | Tipo | Uso |
|--------|------|-----|
| time | f32 | Tiempo global de animación |
| camera_pos | Vec3 | Cálculo de Fresnel/Corona |
| temp_kelvin | f32 | Control de color por temperatura (3000–12000K) |
| disp_amp | f32 | Amplitud del desplazamiento de vértices |
| disp_freq | f32 | Frecuencia espacial del ruido de distorsión |
| speed | f32 | Velocidad de evolución de los ruidos |

Funciones de shader
| Función | Descripción |
|---------|-------------|
| fbm(x,y,oct) | Turbulencia multi‑octava |
| ridge = 1 - abs(2*noise-1) | Crestas para picos energéticos |
| cellular(x,y) | Granulación/plasma |
| kelvin_to_rgb(T) | Mapea temperatura a color |
| star_displacement(n,u) | Desplazamiento radial de vértices |
| star_color(frag,u) | Color final con intensidad, corona y temperatura |

Ejecución
```powershell
cargo run --bin starlab --release
```

Criterios cubiertos
- Creatividad/realismo: granulación + corona + picos con ridge y cellular.
- Complejidad: combinación fbm + ridge + cellular con parámetros ajustables.
- Tiempo/animación: uniform time y speed con pulsación suave.
- Ruido: value noise + ridge + cellular sin texturas.
- Emisión variable y flare: desplazamiento radial de vértices.
- Color por temperatura: kelvin_to_rgb y mezcla dependiente de intensidad.

Resumen técnico
- Rasterizador por software: NDC/viewport, z‑buffer, barycentric fill.
- Shaders: normal desde posición de esfera, rim/corona y mezcla de capas.
- Ruido: value noise 2D, fBM, ridge (1‑|2n‑1|), cellular tipo Worley 2D.

Capturas
| Elemento | Imagen |
|----------|--------|
| Estrella Procedural | ![star](planetLab/docs/star.gif) |
