# Graficas_Proy3
## Proyecto 3 de Gráficas por computadora - Space Travel

---

## Descripción

Este proyecto implementa un sistema solar interactivo renderizado en Rust, donde puedes explorar un sistema solar ficticio con planetas orbitando un sol, y manipular una cámara para obtener diferentes vistas. Además, puedes mover una nave espacial en la órbita de la Tierra y alternar entre una vista estándar y una vista aérea ("Bird's Eye View").

---

### Video demostrativo

[![Watch the video](https://img.youtube.com/vi/jJR7EOxyCRU/0.jpg)](https://youtu.be/jJR7EOxyCRU)

---

## 🛠️ Requisitos del sistema
Dependencias
Este proyecto utiliza varias bibliotecas de Rust para el manejo de gráficos, entrada de usuario, y ruido. Asegúrate de incluir las siguientes dependencias en tu archivo Cargo.toml:

```toml
[dependencies]
nalgebra-glm = "0.15"        # Librería para matemáticas 3D (vectores, matrices, etc.)
minifb = "0.18.0"           # Ventana gráfica simple
winit = "0.28.6"            # Entrada de usuario avanzada (mouse y teclado)
fastnoise-lite = "0.4.0"    # Generación de ruido procedural
image = "0.24.5"            # Manejo de texturas e imágenes
once_cell = "1.17.2"        # Singleton para texturas y normal maps
rand = "0.8.5"              # Generador de números aleatorios`
```

---

## 🚀 Cómo correr el proyecto

Clonar el repositorio:
```bash
git clone <https://github.com/nel-eleven11/Graficas_Proy3>
cd <Graficas_Proy3>
```

Compilar y ejecutar:
Asegúrate de tener instalado el compilador de Rust. Si no lo tienes, instálalo desde rustup.
```bash
cargo run --release
```

Controles disponibles:

Teclado:
- W, A, S, D: Rotar la cámara alrededor del sistema solar.
- Q, E: Mover la cámara hacia arriba/abajo.
- J, K, I, L: Mover la nave espacial.
- B: Activar/desactivar la vista aérea (Bird's Eye View).
- Esc: Salir del programa.

Mouse:
- Arrastrar el mouse mientras presionas el botón izquierdo para rotar la cámara.
- Usa el scroll para acercar/alejar (zoom).



