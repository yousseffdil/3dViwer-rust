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
```

Ejemplo con animación:

```bash
./showObj -m cube.obj --rotate
```

## Ejemplo visual

<video src="./sources/video.mp4" controls>
</video>


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

- Soporte para zoom interactivo.
- Control de cámara desde teclado.
- Más opciones de shading (sombreado por intensidad).
- Exportar la animación a un GIF o video ASCII.

## 📜 Licencia

Este proyecto está bajo la licencia MIT.  
¡Siéntete libre de usarlo, mejorarlo y compartirlo!

---
Made with ❤️ in Rust 🦀
