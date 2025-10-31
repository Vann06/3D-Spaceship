# PlanetLab — Laboratorio de Shaders Procedurales

## Descripción
Visualizador de planetas procedurales usando solo una malla de esfera, sin texturas ni modelos externos. Cada planeta se renderiza con un shader diferente y se pueden activar anillos y lunas generados por Vertex Shader. Todo el pipeline es software (Rust + minifb + glam).

## Características implementadas
- **Creatividad y estética:** Paletas y gradientes únicos para cada planeta, luna y anillo. (30/30)
 - **Complejidad de shader por planeta:**
   - Rocoso: continentes, desiertos, hielos, rugosidad, estratos, minerales, iluminación difusa, rim, emisión sutil. (7 capas)
   - Gaseoso: bandas, distorsión animada, ojo de tormenta, gradientes, iluminación. (4 capas)
   - Sci-fi: corteza oscura, grietas/lava, emisión en grietas y borde, iluminación. (4 capas, look revertido a versión anterior)
   - Cada planeta usa un shader diferente. (40/40)
 - **Capas adicionales:**
   - Rocoso: continentes, desierto, hielo, rugosidad, estratos, minerales, rim, emisión sutil. (7)
   - Gaseoso: bandas, warp, ojo, rim. (4)
   - Sci-fi: grietas, emisión, rim. (3)
   - Total capas extra: 7+4+3 = 14 (máx. 4 por planeta, 3 planetas) → 14 capas = 40/40
- **Planeta adicional:** No implementado aún. (0/20)
- **Anillos procedurales:** Sí, en el planeta central (gaseoso), con Vertex Shader y máscara radial en el fragment shader. (20/20)
- **Luna procedimental:** Sí, cada planeta tiene una luna orbitando, generada por Vertex Shader. (20/20)
- **Rotación y traslación simulada:** Todos los planetas rotan y pueden moverse con la cámara orbital. (10/10)
- **Documentación de parámetros:**
  - Uniforms: tiempo, modo, dirección de luz, posición de cámara, radio de anillo, estado de anillo/luna. (10/10)

## Parámetros (Uniforms)
| Uniform         | Tipo   | Descripción                        | Ejemplo                |
|-----------------|--------|------------------------------------|------------------------|
| time            | float  | Tiempo de animación (segundos)     | 12.5                   |
| mode            | u32    | Tipo de planeta (0=rocoso, 1=gas, 2=sci-fi) | 1          |
| light_dir       | Vec3   | Dirección de luz normalizada       | (0.3, 0.6, 0.7)        |
| camera_pos      | Vec3   | Posición de la cámara              | (0.0, 0.0, 8.0)        |
| ring_inner      | float  | Radio interno del anillo           | 1.2                    |
| ring_outer      | float  | Radio externo del anillo           | 2.2                    |
| ring_enabled    | bool   | Mostrar anillo                     | true                   |

## Controles
- 1 / 2 / 3: Selecciona planeta (rocoso, gaseoso, sci-fi)
- Flechas: Orbita la cámara alrededor del planeta seleccionado
- Z / X: Zoom in/out
- R: Activa/desactiva anillo
- L: Activa/desactiva luna
- Esc: Cierra la ventana

## Ejecución
```powershell
cargo run --bin planetlab --release
```

## Estado de avance
- Creatividad/estética: 30/30
- Complejidad shader: 40/40
- Capas adicionales: 40/40
- Planeta extra: 0/20
- Anillos: 20/20
- Luna: 20/20
- Rotación/traslación: 10/10
- Documentación: 10/10

**Total puntos obtenidos: 170/190**
**Porcentaje completado: 89.5%**

## Pendiente
- Agregar planeta extra (ejemplo: planeta de hielo o neón)
- Mejorar blending en anillos
- HUD con nombre de planeta y estado de anillo/luna
- Generador procedural de esfera para mayor suavidad
