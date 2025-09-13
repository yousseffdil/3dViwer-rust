# ShowObj: Visualizador de Wireframes 3D en el Terminal

ShowObj es una herramienta de línea de comandos (CLI) desarrollada en Rust que renderiza modelos 3D en formato de archivo `.obj` como wireframes directamente en tu terminal. Ofrece la posibilidad de visualizar modelos estáticos o animados con una rotación continua.

## Descripción del Funcionamiento

El script lleva a cabo una serie de pasos para lograr la visualización del modelo 3D en el terminal:

1.  **Análisis de Argumentos de Línea de Comandos**: Utiliza la librería `clap` para procesar los argumentos proporcionados por el usuario, como la ruta al archivo del modelo y si debe rotar. [16, 27]
2.  **Carga del Modelo .obj**: Lee un archivo en formato `.obj`, un estándar en gráficos 3D que almacena la geometría del objeto, incluyendo vértices y caras. [1, 9, 13, 18] La aplicación extrae las coordenadas de los vértices (puntos en el espacio 3D) y las caras (los polígonos que conectan los vértices).
3.  **Normalización del Modelo**: Para asegurar que el modelo se ajuste correctamente a la pantalla del terminal, independientemente de su tamaño y posición originales, se aplica un proceso de normalización. Este proceso centra el modelo en el origen de coordenadas (0,0,0) y lo escala para que quepa dentro de un cubo unitario.
4.  **Rotación (Opcional)**: Si se activa la opción de rotación, el script aplica continuamente rotaciones al modelo alrededor de los ejes X, Y y Z, creando un efecto de animación.
5.  **Proyección 3D a 2D**: Los vértices 3D del modelo se proyectan sobre un plano 2D para poder representarlos en la pantalla del terminal, que es bidimensional. [30, 41]
6.  **Dibujo de Líneas**: Utiliza el algoritmo de Bresenham para dibujar las líneas que conectan los vértices proyectados, formando así el wireframe del modelo. [2, 5, 6, 7]
7.  **Renderizado en el Terminal con Colores**: Las líneas del wireframe se renderizan en el terminal utilizando caracteres. La librería `colored` se emplea para asignar colores a estos caracteres en función de la profundidad (coordenada Z) de los puntos, creando un efecto de gradiente que da una sensación de profundidad. [28, 34, 35]

## Fórmulas Utilizadas

El script emplea varias fórmulas matemáticas fundamentales en los gráficos por computadora:

### Rotación 3D

Para rotar un punto (o vértice) en un espacio 3D alrededor de los ejes principales (X, Y, Z), se utilizan las siguientes matrices de rotación: [15, 36, 38, 43, 46]

*   **Rotación alrededor del eje X**:
    ```
    x' = x
    y' = y * cos(θ) - z * sin(θ)
    z' = y * sin(θ) + z * cos(θ)
    ```

*   **Rotación alrededor del eje Y**:
    ```
    x' = x * cos(θ) + z * sin(θ)
    y' = y
    z' = -x * sin(θ) + z * cos(θ)
    ```

*   **Rotación alrededor del eje Z**:
    ```
    x' = x * cos(θ) - y * sin(θ)
    y' = x * sin(θ) + y * cos(θ)
    z' = z
    ```
    Donde `(x, y, z)` son las coordenadas originales del vértice, `(x', y', z')` son las coordenadas después de la rotación y `θ` es el ángulo de rotación.

### Normalización del Modelo

La normalización se realiza en dos pasos:

1.  **Cálculo del Bounding Box y el Centro**: Se encuentran las coordenadas mínimas y máximas (min_x, max_x, min_y, max_y, min_z, max_z) para determinar el tamaño del modelo. El centro se calcula como:
    *   `centro_x = (min_x + max_x) / 2`
    *   `centro_y = (min_y + max_y) / 2`
    *   `centro_z = (min_z + max_z) / 2`

2.  **Traslación y Escalado**: Cada vértice se traslada para que el centro del modelo coincida con el origen y luego se escala dividiendo por la dimensión más grande del bounding box (`max_size`):
    *   `v.x = (v.x - centro_x) / max_size`
    *   `v.y = (v.y - centro_y) / max_size`
    *   `v.z = (v.z - centro_z) / max_size`

### Proyección a 2D

El script utiliza una proyección ortográfica simple para convertir las coordenadas 3D en coordenadas de pantalla 2D. Esta proyección se combina con un escalado y un centrado en la pantalla: [40]

*   `pantalla_x = (v.x * escala) + (ancho_pantalla / 2)`
*   `pantalla_y = (v.y * escala) + (alto_pantalla / 2)`

### Interpolación Lineal (Lerp) para el Color

Para determinar el color de cada punto en una línea entre dos vértices, se utiliza la interpolación lineal (lerp) sobre sus valores de profundidad (coordenada Z):

*   `z_actual = z1 + (z2 - z1) * t`
    Donde `z1` y `z2` son los valores de profundidad de los dos vértices y `t` es un valor entre 0 y 1 que representa la posición a lo largo de la línea.

## README para Descargar

```markdown
# ShowObj: Visualizador de Wireframes 3D en el Terminal

Una herramienta de línea de comandos escrita en Rust para renderizar y visualizar modelos 3D en formato `.obj` como wireframes directamente en tu terminal.

## Características

*   Carga y muestra modelos 3D desde archivos `.obj`.
*   Renderiza los modelos como wireframes.
*   Opción para animar el modelo con una rotación continua.
*   Utiliza un gradiente de color para simular la profundidad.
*   Normaliza automáticamente los modelos para un ajuste y visualización óptimos.

## Requisitos Previos

*   Tener instalado el compilador de Rust y Cargo. Puedes instalarlos siguiendo las instrucciones en [rustup.rs](https://rustup.rs/).

## Instalación

1.  Clona este repositorio o descarga los archivos.
2.  Navega al directorio del proyecto en tu terminal.
3.  Compila el proyecto ejecutando:
    ```bash
    cargo build --release
    ```
    El ejecutable se encontrará en `target/release/showObj`.

## Uso

Para visualizar un modelo 3D, ejecuta el siguiente comando:

```bash
./target/release/showObj --model RUTA_A_TU_MODELO.obj
```

## Opciones
- ```-m, --model <RUTA> ```: Especifica la ruta al archivo del modelo .obj que deseas visualizar.
- ```--rotate```: Activa la rotación automática del modelo.
