# showObj

**showObj** es una herramienta de línea de comandos (CLI) escrita en Rust que permite visualizar wireframes de modelos 3D en formato `.obj` directamente en el terminal.

## Características

- Carga y muestra modelos `.obj` en modo wireframe.
- Normaliza y centra el modelo para una visualización adecuada.
- Opción de rotación automática para ver el modelo desde diferentes ángulos.
- Renderizado en ASCII en el terminal.

## Instalación

1. **Clona el repositorio:**
   ```bash
   git clone <URL_DEL_REPOSITORIO>
   cd showObj
   ```

2. **Compila el proyecto:**
   ```bash
   cargo build --release
   ```

3. **Ejecuta el binario:**
   ```bash
   ./target/release/showObj --model ruta/al/modelo.obj
   ```

## Uso
