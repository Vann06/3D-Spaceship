# PlanetLab — Paletas Creativas y Anillo Procedural

## Descripción
Visualizador de 5 planetas procedurales Rocoso Bio-Lum, Gaseoso Turquesa/Lima, Alien Verde-Púrpura, Lava Incandescente, Hielo Cristal y cambiando el shader activo con teclas 1–5. Incluye luna orbital y anillo procedural.

## Características implementadas
## Paletas y Capas 
| Planeta | Key | Capas Principales | Colores | Emisión |
|---------|-----|-------------------|---------|---------|
| Rocoso Bio-Lum | 1 | Continentes, desierto, hielo, rugosidad, estratos, minerales, rim, bioluminiscencia pulsante | Teal, verde musgo, púrpura tenue | Zonas frías + minerales |
| Gaseoso Turquesa/Lima | 2 | Bandas animadas, warp, ojo púrpura, turbulencia fina, rim pulsante | Turquesa, lima, púrpura acento | Pulso en rim y anillo |
| Alien Verde-Púrpura | 3 | Base oscura, grietas, venas verdes/púrpuras, mezcla animada, rim | Verde brillante, púrpura energético | Pulso rápido bioluminiscente |
| Lava Incandescente | 4 | Flujo animado, corteza vs lava, grietas, gradiente térmico, emisión pulsante | Rojo oscuro, naranja, amarillo | Grietas y pulsos térmicos |
| Hielo Cristal | 5 | Variación hielo, grietas, cristales, nieve, aurora, rim | Azul profundo, cian, blanco, verde/ púrpura aurora | Auroras + rim frío |

### Anillo Mejorado (Solo Key 2 — Gigante Gaseoso)
Características: gradiente triple turquesa→lima→púrpura, granulado fbm multi-octava, modulación angular (seno(ángulo+tiempo)) y pulso de brillo.

## Parámetros (Uniforms) y Funciones
| Nombre | Tipo | Uso | Planetas que lo usan |
|--------|------|-----|----------------------|
| time | f32 | Tiempo global animación | Todos |
| mode | u32 | Selección de shader (0..4) | Todos |
| light_dir | Vec3 | Dirección de luz para Lambert y rim | Todos |
| camera_pos | Vec3 | Cálculo de rim (vista) | Todos |
| ring_inner / ring_outer | f32 | Radios para máscara de anillo | Gaseoso (2) |
| ring_enabled | bool | Activa máscara anillo | Gaseoso |
| (interno) fbm() | f32 acumulado | Terreno, bandas, lava flow, hielo, aurora | Según planeta |
| (interno) cracks / masks | f32 | Grietas (lava, alien) | Lava, Alien |
| (interno) emission | f32 | Intensidad lumínica local | Todos (varía) |

### Funciones Clave
| Función | Descripción | Planetas que lo usan |
|---------|-------------|----------------------|
| fbm(x,y,oct) | Ruido fractal 2D multi-octava para variaciones multi-escala y detalle procedimental | Terreno, bandas, lava flow, hielo, aurora |
| lighting(n,view) | Calcula (lambert, rim) — lambert para iluminación difusa y 'rim' para halo de borde — usado en sombreado base | Todos |
| ring logic | Mezcla de bandas radiales + fbm + variación angular (seno sobre el ángulo + tiempo) para colorear el anillo dinámicamente | Gaseoso (Key 2) |
| Pulsos (sin(time * factor)) | Genera señales oscilantes para bioluminiscencia, pulsos térmicos y variaciones rítmicas | Rocky, Alien, Lava, Ice |

## Controles
- 1..5: Cambia planeta activo (una sola esfera)
- Flechas: Orbita la cámara alrededor del planeta seleccionado
- Z / X: Zoom in/out
- R: Activa/desactiva anillo
- L: Activa/desactiva luna
- Esc: Cierra la ventana

## Ejecución
```powershell
cargo run --bin planetlab --release
```

## Notas
- Solo UNA malla (sphere.obj) reutilizada para los 5 planetas.
- Sin texturas: todo color/emisión procedimental.
- Fácil expansión: añade nuevos modos en `shader.rs` (match mode).


## Capturas
| Planeta | Imagen |
|---------|--------|
| Rocoso Bio-Lum | ![rocky](planetLab/docs/rocky.png) |
| Gaseoso Turq-Lima | ![gas](planetLab/docs/gas.png) |
| Alien Verde-Púrpura | ![alien](planetLab/docs/alien.gif) |
| Lava Incandescente | ![lava](planetLab/docs/lava.png) |
| Hielo Cristal | ![ice](planetLab/docs/ice.png) |
