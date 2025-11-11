# StarLab — Estrella Sol


Capturas
| Elemento | Imagen |
|----------|--------|
| Estrella Procedural | ![star](planetLab/docs/star.gif) |


Características clave
- Animación continua basada en time y funciones de ruido (fbm, ridge, cellular/Worley‑like).
- Emisión variable y corona con efecto rim^3 y picos energéticos.
- Desplazamiento radial de vértices (flare) por combinación de ruidos.
- Color controlado por temperatura con kelvin_to_rgb (rojo→blanco azulado).
- Selección dinámica de tipo de ruido: Perlin, Simplex o Cellular.

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
cargo run --bin starlab
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

## Objetivo
Diseñar una estrella o “sol” utilizando shaders y funciones de ruido (Perlin, Simplex o Cellular). La estrella debe mostrar animación con el paso del tiempo, simulando turbulencias, actividad solar o pulsaciones en su superficie.

## Restricciones técnicas
- Únicamente se usa una esfera como base de la estrella.
- No se usan texturas ni materiales precargados.
- La animación se genera con una variable de tiempo (`uniform float time`) y funciones de ruido dentro del shader.
- La apariencia y animación se modifican exclusivamente mediante shaders.
- La animación es continua y cíclica.


## Ruido utilizado y verificación
- Se usa explícitamente Cellular noise (tipo Worley) implementado en `planetLab/src/shader.rs` como `fn cellular(x: f32, y: f32) -> f32`. Esto cumple el requisito de “usar Perlin, Simplex o Cellular”.
- Además, se combina con value noise y fBM para mayor complejidad. La intensidad final se calcula en `star_color` como una mezcla: `intensity = (0.5*base_fbm + 0.3*ridge + 0.2*cell) * (0.85 + 0.30*pulse)`; donde `ridge = 1 - abs(2*noise-1)` y `cell = cellular(...)`.
- Cómo afecta el color/intensidad:
	- La intensidad modula un blending hacia un “hot white” en zonas más energéticas, elevando la emisión aparente.
	- El término `cellular` aporta granulación y picos que alimentan la corona y los “spikes” energéticos.
	- El color base viene de `kelvin_to_rgb(temp_kelvin)` y se mezcla más hacia blanco conforme sube la intensidad.
- Parámetros ajustables (uniforms): `disp_freq` (frecuencia espacial del ruido), `disp_amp` (amplitud de flare), `speed` (velocidad temporal), `temp_kelvin` (gradiente de temperatura).

## Criterios de evaluación 
| Criterio | Cómo se cumple |
|---|---|
| Creatividad visual y realismo | Granulación celular, corona rim^3, picos y pulsación senoidal |
| Complejidad del shader | Combinación fbm + ridge + cellular con parámetros ajustables |
| Tiempo y animación continua | Uso de `time` y `speed` en todas las capas de ruido |
| Perlin/Simplex/Cellular | Cambiable en tiempo real: 1=Perlin, 2=Simplex, 3=Cellular |
| Emisión variable | Intensidad modula mezcla hacia blanco caliente y corona |
| Distorsión/flare por VS | Desplazamiento radial `star_displacement` basado en ruido |
| Color por intensidad/temperatura | `kelvin_to_rgb` + mezcla dependiente de `intensity` |
| Documentación | Uniforms, funciones y verificación aquí en README |

## Controles
- 1 / 2 / 3: Cambiar tipo de ruido (Perlin / Simplex / Cellular)
- Flechas Arriba/Abajo: Aumentar/Reducir amplitud de distorsión (disp_amp)
- Flechas Izquierda/Derecha: Reducir/Aumentar frecuencia espacial (disp_freq)
- W / S: Aumentar/Reducir temperatura (temp_kelvin)
- A / D: Reducir/Aumentar velocidad (speed)

<!-- Inserta un video local (MP4) -->
<video src="planetLab/docs/star.mp4" controls autoplay loop muted playsinline style="max-width:100%; height:auto;">
  Tu navegador no soporta la etiqueta <code>video</code>. <a href="planetLab/docs/star.mp4">Descarga el video</a>.
</video>

