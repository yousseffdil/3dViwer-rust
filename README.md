# showObj

Una herramienta CLI escrita en **Rust** para mostrar wireframes de modelos 3D (`.obj`) directamente en tu terminal, con colores y animación opcional de rotación.

## Características

- Renderizado de **wireframes** en ASCII.
- **Gradiente de colores** según la profundidad (`z-value`).
- **Animación con rotación** en los tres ejes (`--rotate`).
- Soporte para archivos `.obj` con **vértices** y **caras**.

## Instalación

Asegúrate de tener instalado [Rust](https://www.rust-lang.org/tools/install).

Clona el repositorio y compila:

```bash
git clone https://github.com/tuusuario/showObj.git
cd showObj
cargo build --release
```

El binario quedará en `target/release/showObj`.

## Uso

Ejecuta el programa pasando un archivo `.obj`:

```bash
./showObj -m modelo.obj
```
## Opciones disponibles:

```
Uso: showObj [OPTIONS] --model <MODELO>

Opciones:
  -m, --model <MODELO>   Ruta al archivo .obj a cargar
      --rotate           Habilita la animación con rotación
  -h, --help             Muestra la ayuda
  -V, --version          Muestra la versión
  -w, --wireframe        Muestra el viwe en formato wireframe, Por default muestra un viewPort lleno
  --arrows               Permite mover el modelo 3d con las flechas del teclado ( Controles: ← → ↑ ↓ para rotar | A/D para rotar en Z | +/- para zoom | ESC/Q para salir )
```

Ejemplo con animación:

```bash
./showObj -m cube.obj --rotate
```

## Ejemplo visual
<img src="https://github.com/user-attachments/assets/5c096213-084e-4c09-8683-300c11a465f8">
## Estructura de proyecto

```
src/
├── main.rs        # Lógica principal del CLI
├── obj_loader.rs  # Lectura de archivos .obj
├── render.rs      # Renderizado ASCII con colores
├── transform.rs   # Rotaciones y proyecciones
```

## 🛠 Tecnologías utilizadas

- [Rust](https://www.rust-lang.org/)
- [Clap](https://docs.rs/clap/latest/clap/) → Parsing de argumentos
- [Colored](https://docs.rs/colored/latest/colored/) → Colores en terminal

## Ideas futuras

- Soporte para zoom interactivo. ❤️
- Control de cámara desde teclado. ❤️
- Más opciones de shading (sombreado por intensidad).
- Exportar la animación a un GIF o video ASCII.

## Mejoras
- [ ] Zero-Allocation
- [ ] Buffer plano y uso de `char` en vez de `String`
- [ ] Trigonometría precomputada.
- [ ] Early reject en rasterizado.
- [ ] Preparación para paralelismo con `rayon`
## 📜 Licencia

Este proyecto está bajo la licencia MIT.  
¡Siéntete libre de usarlo, mejorarlo y compartirlo!

---
Made with ❤️ in Rust 🦀
