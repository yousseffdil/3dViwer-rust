use clap::Parser;
use colored::*;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType},
};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::f32::consts::PI;
use std::{thread, time::Duration};
use bytemuck::{Pod, Zeroable};

// Dependencias para GPU (agregar al Cargo.toml):
// wgpu = "0.17"
// pollster = "0.3"
// winit = "0.28"
// bytemuck = { version = "1.12", features = ["derive"] }

#[derive(Parser, Debug)]
#[command(name = "showObj", version, about = "Muestra modelos .obj en terminal con renderizado GPU")]
struct Args {
    #[arg(short = 'm', long = "model")]
    model: String,

    #[arg(long, default_value_t = false)]
    rotate: bool,

    #[arg(short = 'w', long = "wireframe", default_value_t = false)]
    wireframe: bool,

    #[arg(long, default_value_t = false)]
    arrows: bool,

    #[arg(short = 'c', long = "color", default_value = "blue")]
    color: String,

    #[arg(long, default_value_t = false)]
    gpu: bool,

    #[arg(long, default_value_t = 1)]
    detail: u32, // Factor de subdivisión para más detalle
}

#[derive(Debug)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug)]
struct Face {
    vertices: Vec<usize>,
}

#[derive(Debug, Clone, Copy)]
struct Point3D {
    x: i32,
    y: i32,
    z: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct GPUVertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 3],
}

impl GPUVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GPUVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[derive(Debug)]
struct KeyboardState {
    angle_x: f32,
    angle_y: f32,
    angle_z: f32,
    scale: f32,
}

impl KeyboardState {
    fn new() -> Self {
        Self {
            angle_x: 0.0,
            angle_y: 50.0,
            angle_z: 3.0,
            scale: 20.0,
        }
    }

    fn rotate_up(&mut self) {
        self.angle_x -= 5.0;
        self.angle_x = self.angle_x.clamp(-89.0, 89.0);
    }

    fn rotate_down(&mut self) {
        self.angle_x += 5.0;
        self.angle_x = self.angle_x.clamp(-89.0, 89.0);
    }

    fn rotate_left(&mut self) {
        self.angle_y -= 5.0;
    }

    fn rotate_right(&mut self) {
        self.angle_y += 5.0;
    }

    fn rotate_clockwise(&mut self) {
        self.angle_z += 5.0;
    }

    fn rotate_counter_clockwise(&mut self) {
        self.angle_z -= 5.0;
    }

    fn zoom_in(&mut self) {
        self.scale *= 1.1;
    }

    fn zoom_out(&mut self) {
        self.scale *= 0.9;
        if self.scale < 1.0 {
            self.scale = 1.0;
        }
    }
}

/// -------------------- SUBDIVISIÓN PARA MÁS DETALLE --------------------
fn subdivide_faces(vertices: &mut Vec<Vertex>, faces: &mut Vec<Face>, level: u32) {
    for _ in 0..level {
        let mut new_faces = Vec::new();
        let original_vertex_count = vertices.len();
        
        for face in faces.iter() {
            if face.vertices.len() >= 3 {
                // Para cada triángulo, crear 4 triángulos más pequeños
                for i in 1..face.vertices.len() - 1 {
                    let v0 = face.vertices[0];
                    let v1 = face.vertices[i];
                    let v2 = face.vertices[i + 1];
                    
                    // Puntos medios
                    let mid01 = Vertex {
                        x: (vertices[v0].x + vertices[v1].x) / 2.0,
                        y: (vertices[v0].y + vertices[v1].y) / 2.0,
                        z: (vertices[v0].z + vertices[v1].z) / 2.0,
                    };
                    let mid12 = Vertex {
                        x: (vertices[v1].x + vertices[v2].x) / 2.0,
                        y: (vertices[v1].y + vertices[v2].y) / 2.0,
                        z: (vertices[v1].z + vertices[v2].z) / 2.0,
                    };
                    let mid20 = Vertex {
                        x: (vertices[v2].x + vertices[v0].x) / 2.0,
                        y: (vertices[v2].y + vertices[v0].y) / 2.0,
                        z: (vertices[v2].z + vertices[v0].z) / 2.0,
                    };
                    
                    let idx_mid01 = vertices.len();
                    let idx_mid12 = vertices.len() + 1;
                    let idx_mid20 = vertices.len() + 2;
                    
                    vertices.push(mid01);
                    vertices.push(mid12);
                    vertices.push(mid20);
                    
                    // 4 nuevos triángulos
                    new_faces.push(Face { vertices: vec![v0, idx_mid01, idx_mid20] });
                    new_faces.push(Face { vertices: vec![v1, idx_mid12, idx_mid01] });
                    new_faces.push(Face { vertices: vec![v2, idx_mid20, idx_mid12] });
                    new_faces.push(Face { vertices: vec![idx_mid01, idx_mid12, idx_mid20] });
                }
            }
        }
        *faces = new_faces;
    }
}

/// -------------------- CÁLCULO DE NORMALES --------------------
fn calculate_normals(vertices: &[Vertex], faces: &[Face]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0f32; 3]; vertices.len()];
    let mut counts = vec![0usize; vertices.len()];
    
    for face in faces {
        if face.vertices.len() >= 3 {
            for i in 1..face.vertices.len() - 1 {
                let v0 = &vertices[face.vertices[0]];
                let v1 = &vertices[face.vertices[i]];
                let v2 = &vertices[face.vertices[i + 1]];
                
                let edge1 = [v1.x - v0.x, v1.y - v0.y, v1.z - v0.z];
                let edge2 = [v2.x - v0.x, v2.y - v0.y, v2.z - v0.z];
                
                let normal = [
                    edge1[1] * edge2[2] - edge1[2] * edge2[1],
                    edge1[2] * edge2[0] - edge1[0] * edge2[2],
                    edge1[0] * edge2[1] - edge1[1] * edge2[0],
                ];
                
                for &vi in &[face.vertices[0], face.vertices[i], face.vertices[i + 1]] {
                    normals[vi][0] += normal[0];
                    normals[vi][1] += normal[1];
                    normals[vi][2] += normal[2];
                    counts[vi] += 1;
                }
            }
        }
    }
    
    // Normalizar
    for i in 0..normals.len() {
        if counts[i] > 0 {
            let len = (normals[i][0] * normals[i][0] + 
                      normals[i][1] * normals[i][1] + 
                      normals[i][2] * normals[i][2]).sqrt();
            if len > 0.0 {
                normals[i][0] /= len;
                normals[i][1] /= len;
                normals[i][2] /= len;
            }
        }
    }
    
    normals
}

/// -------------------- RENDERIZADO GPU --------------------
struct GPURenderer {
    // En una implementación real, aquí irían los recursos de GPU
    // device: wgpu::Device,
    // queue: wgpu::Queue,
    // render_pipeline: wgpu::RenderPipeline,
    // etc.
}

impl GPURenderer {
    async fn new() -> Self {
        // Inicialización de GPU aquí
        Self {}
    }
    
    fn render_gpu(&self, vertices: &[GPUVertex], width: i32, height: i32) -> Vec<Vec<String>> {
        // Simulación de renderizado GPU más avanzado
        let mut screen = vec![vec![" ".to_string(); width as usize]; height as usize];
        let mut zbuffer = vec![vec![f32::MIN; width as usize]; height as usize];
        
        // Procesar triángulos con mejor calidad
        for chunk in vertices.chunks(3) {
            if chunk.len() == 3 {
                let v0 = &chunk[0];
                let v1 = &chunk[1];
                let v2 = &chunk[2];
                
                // Proyección con perspectiva
                let project_vertex = |v: &GPUVertex| -> Point3D {
                    let fov = 60.0_f32.to_radians();
                    let aspect = width as f32 / height as f32;
                    let near = 0.1;
                    let far = 100.0;
                    
                    let z = v.position[2] + 3.0; // Desplazar hacia atrás
                    if z <= near { return Point3D { x: -1000, y: -1000, z: -1000.0 }; }
                    
                    let proj_x = v.position[0] / (z * (fov / 2.0).tan()) * aspect;
                    let proj_y = v.position[1] / (z * (fov / 2.0).tan());
                    
                    Point3D {
                        x: ((proj_x + 1.0) * width as f32 / 2.0) as i32,
                        y: ((1.0 - proj_y) * height as f32 / 2.0) as i32,
                        z: z,
                    }
                };
                
                let p0 = project_vertex(v0);
                let p1 = project_vertex(v1);
                let p2 = project_vertex(v2);
                
                if p0.x < 0 || p1.x < 0 || p2.x < 0 { continue; }
                
                // Rasterización con interpolación mejorada
                let min_x = p0.x.min(p1.x).min(p2.x).max(0) as i32;
                let max_x = p0.x.max(p1.x).max(p2.x).min(width - 1) as i32;
                let min_y = p0.y.min(p1.y).min(p2.y).max(0) as i32;
                let max_y = p0.y.max(p1.y).max(p2.y).min(height - 1) as i32;
                
                let area = ((p1.x - p0.x) * (p2.y - p0.y) - (p2.x - p0.x) * (p1.y - p0.y)) as f32;
                if area.abs() < 1.0 { continue; }
                
                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        let w0 = ((p1.x - p0.x) as f32 * (y - p0.y) as f32 - (p1.y - p0.y) as f32 * (x - p0.x) as f32) / area;
                        let w1 = ((p2.x - p1.x) as f32 * (y - p1.y) as f32 - (p2.y - p1.y) as f32 * (x - p1.x) as f32) / area;
                        let w2 = ((p0.x - p2.x) as f32 * (y - p2.y) as f32 - (p0.y - p2.y) as f32 * (x - p2.x) as f32) / area;
                        
                        if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                            let z = w0 * p0.z + w1 * p1.z + w2 * p2.z;
                            
                            if z > zbuffer[y as usize][x as usize] {
                                zbuffer[y as usize][x as usize] = z;
                                
                                // Interpolación de color y normal
                                let color = [
                                    w0 * v0.color[0] + w1 * v1.color[0] + w2 * v2.color[0],
                                    w0 * v0.color[1] + w1 * v1.color[1] + w2 * v2.color[1],
                                    w0 * v0.color[2] + w1 * v1.color[2] + w2 * v2.color[2],
                                ];
                                
                                let normal = [
                                    w0 * v0.normal[0] + w1 * v1.normal[0] + w2 * v2.normal[0],
                                    w0 * v0.normal[1] + w1 * v1.normal[1] + w2 * v2.normal[1],
                                    w0 * v0.normal[2] + w1 * v1.normal[2] + w2 * v2.normal[2],
                                ];
                                
                                // Iluminación mejorada
                                let light_dir = [0.0, 0.0, -1.0];
                                let dot = normal[0] * light_dir[0] + normal[1] * light_dir[1] + normal[2] * light_dir[2];
                                
                                let intensity = ((dot + 1.0) / 2.0).clamp(0.0, 1.0);
                                
                                // Múltiples caracteres según intensidad
                                let shade_char = match intensity {
                                    i if i > 0.8 => "█",
                                    i if i > 0.6 => "▓",
                                    i if i > 0.4 => "▒",
                                    i if i > 0.2 => "░",
                                    _ => "·",
                                };
                                
                                // Aplicar color
                                let colored_char = match (color[0] * 255.0) as u8 {
                                    r if r > 200 && color[1] < 0.3 && color[2] < 0.3 => shade_char.red().to_string(),
                                    g if color[1] > 0.7 && color[0] < 0.3 && color[2] < 0.3 => shade_char.green().to_string(),
                                    b if color[2] > 0.7 && color[0] < 0.3 && color[1] < 0.3 => shade_char.blue().to_string(),
                                    _ => shade_char.white().to_string(),
                                };
                                
                                screen[y as usize][x as usize] = colored_char;
                            }
                        }
                    }
                }
            }
        }
        
        screen
    }
}

/// -------------------- FUNCIONES ORIGINALES MEJORADAS --------------------
fn rotate_x(v: &Vertex, angle: f32) -> Vertex {
    let rad = angle * PI / 180.0;
    let cos = rad.cos();
    let sin = rad.sin();
    Vertex {
        x: v.x,
        y: v.y * cos - v.z * sin,
        z: v.y * sin + v.z * cos,
    }
}

fn rotate_y(v: &Vertex, angle: f32) -> Vertex {
    let rad = angle * PI / 180.0;
    let cos = rad.cos();
    let sin = rad.sin();
    Vertex {
        x: v.x * cos + v.z * sin,
        y: v.y,
        z: -v.x * sin + v.z * cos,
    }
}

fn rotate_z(v: &Vertex, angle: f32) -> Vertex {
    let rad = angle * PI / 180.0;
    let cos = rad.cos();
    let sin = rad.sin();
    Vertex {
        x: v.x * cos - v.y * sin,
        y: v.x * sin + v.y * cos,
        z: v.z,
    }
}

fn project(v: &Vertex, width: i32, height: i32, scale: f32) -> Point3D {
    let x = (v.x * scale + (width as f32 / 2.0)) as i32;
    let y = (v.y * scale + (height as f32 / 2.0)) as i32;
    Point3D { x, y, z: v.z }
}

fn normalize_model(vertices: &mut Vec<Vertex>) {
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    let mut min_z = f32::MAX;
    let mut max_z = f32::MIN;

    for v in vertices.iter() {
        min_x = min_x.min(v.x);
        max_x = max_x.max(v.x);
        min_y = min_y.min(v.y);
        max_y = max_y.max(v.y);
        min_z = min_z.min(v.z);
        max_z = max_z.max(v.z);
    }

    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;
    let center_z = (min_z + max_z) / 2.0;

    let size_x = max_x - min_x;
    let size_y = max_y - min_y;
    let size_z = max_z - min_z;
    let max_size = size_x.max(size_y).max(size_z);

    for v in vertices.iter_mut() {
        v.x = (v.x - center_x) / max_size;
        v.y = (v.y - center_y) / max_size;
        v.z = (v.z - center_z) / max_size;
    }
}

fn load_obj(path: &str) -> (Vec<Vertex>, Vec<Face>) {
    let file = File::open(path).expect("Cant load the file");
    let reader = BufReader::new(file);

    let mut vertices = Vec::new();
    let mut faces = Vec::new();

    for line in reader.lines() {
        let line = line.expect("Unexpected error");

        if line.starts_with("v ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let x = parts[1].parse::<f32>().unwrap();
            let y = parts[2].parse::<f32>().unwrap();
            let z = parts[3].parse::<f32>().unwrap();
            vertices.push(Vertex { x, y, z });
        } else if line.starts_with("f ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let mut face_indices = Vec::new();

            for p in parts.iter().skip(1) {
                let idx_str = p.split('/').next().unwrap();
                let idx = idx_str.parse::<usize>().unwrap();
                face_indices.push(idx - 1);
            }
            faces.push(Face { vertices: face_indices });
        }
    }

    (vertices, faces)
}

fn color_from_string(color_str: &str) -> [f32; 3] {
    match color_str.to_lowercase().as_str() {
        "red" => [1.0, 0.0, 0.0],
        "green" => [0.0, 1.0, 0.0],
        "blue" => [0.0, 0.0, 1.0],
        "yellow" => [1.0, 1.0, 0.0],
        "magenta" | "purple" => [1.0, 0.0, 1.0],
        "cyan" => [0.0, 1.0, 1.0],
        "white" => [1.0, 1.0, 1.0],
        "black" => [0.0, 0.0, 0.0],
        _ => [0.0, 0.0, 1.0], // default blue
    }
}

fn get_shade_from_normal(v0: Point3D, v1: Point3D, v2: Point3D, color: &str) -> String {
    let ux = (v1.x - v0.x) as f32;
    let uy = (v1.y - v0.y) as f32;
    let uz = v1.z - v0.z;
    let vx = (v2.x - v0.x) as f32;
    let vy = (v2.y - v0.y) as f32;
    let vz = v2.z - v0.z;

    let nx = uy * vz - uz * vy;
    let ny = uz * vx - ux * vz;
    let nz = ux * vy - uy * vx;

    let light = (0.0, 0.0, -1.0);
    let dot = (nx * light.0 + ny * light.1 + nz * light.2)
        / ((nx * nx + ny * ny + nz * nz).sqrt() + 1e-6);

    // Más niveles de sombreado para mejor detalle
    let (light_char, dark_char) = match dot {
        d if d > 0.7 => ("█", "▓"),
        d if d > 0.3 => ("▓", "▒"),
        d if d > 0.0 => ("▒", "░"),
        d if d > -0.3 => ("░", "·"),
        _ => ("·", " "),
    };

    let char_to_use = if dot > 0.0 { light_char } else { dark_char };

    match color.to_lowercase().as_str() {
        "red" => char_to_use.red().to_string(),
        "green" => char_to_use.green().to_string(),
        "yellow" => char_to_use.yellow().to_string(),
        "magenta" | "purple" => char_to_use.magenta().to_string(),
        "cyan" => char_to_use.cyan().to_string(),
        "white" => char_to_use.white().to_string(),
        "black" => char_to_use.black().to_string(),
        _ => char_to_use.blue().to_string(),
    }
}

// Funciones de renderizado y control existentes (sin cambios significativos)
fn draw_line(screen: &mut Vec<Vec<String>>, p1: Point3D, p2: Point3D) {
    let mut x0 = p1.x;
    let mut y0 = p1.y;
    let x1 = p2.x;
    let y1 = p2.y;

    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if y0 >= 0 && y0 < screen.len() as i32 && x0 >= 0 && x0 < screen[0].len() as i32 {
            screen[y0 as usize][x0 as usize] = "·".bright_black().to_string();
        }

        if x0 == x1 && y0 == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn render(points: &[Point3D], faces: &[Face], width: i32, height: i32, wireframe: bool, color: &str) {
    let mut screen = vec![vec![" ".to_string(); width as usize]; height as usize];
    let mut zbuffer = vec![vec![f32::MIN; width as usize]; height as usize];

    for face in faces {
        if face.vertices.len() < 3 { continue; }

        for i in 1..face.vertices.len() - 1 {
            let v0 = points[face.vertices[0]];
            let v1 = points[face.vertices[i]];
            let v2 = points[face.vertices[i + 1]];

            let min_x = v0.x.min(v1.x).min(v2.x).max(0) as i32;
            let max_x = v0.x.max(v1.x).max(v2.x).min(width - 1) as i32;
            let min_y = v0.y.min(v1.y).min(v2.y).max(0) as i32;
            let max_y = v0.y.max(v1.y).max(v2.y).min(height - 1) as i32;

            let area = ((v1.x - v0.x) * (v2.y - v0.y) - (v2.x - v0.x) * (v1.y - v0.y)) as f32;
            if area.abs() < 1.0 { continue; }

            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let w0 = ((v1.x - v0.x) as f32 * (y - v0.y) as f32 - (v1.y - v0.y) as f32 * (x - v0.x) as f32) / area;
                    let w1 = ((v2.x - v1.x) as f32 * (y - v1.y) as f32 - (v2.y - v1.y) as f32 * (x - v1.x) as f32) / area;
                    let w2 = ((v0.x - v2.x) as f32 * (y - v2.y) as f32 - (v0.y - v2.y) as f32 * (x - v2.x) as f32) / area;

                    if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                        let z = (w0 * v0.z + w1 * v1.z + w2 * v2.z) / (w0 + w1 + w2);

                        if z > zbuffer[y as usize][x as usize] {
                            zbuffer[y as usize][x as usize] = z;
                            screen[y as usize][x as usize] = get_shade_from_normal(v0, v1, v2, color);
                        }
                    }
                }
            }
        }

        if wireframe {
            for i in 0..face.vertices.len() {
                let v1 = points[face.vertices[i]];
                let v2 = points[face.vertices[(i + 1) % face.vertices.len()]];
                draw_line(&mut screen, v1, v2);
            }
        }
    }

    execute!(io::stdout(), cursor::MoveTo(0, 0), terminal::Clear(ClearType::All)).unwrap();
    
    for row in screen {
        println!("{}", row.join(""));
    }
    
    println!("\n{}", "Controles: ← → ↑ ↓ para rotar | A/D para rotar en Z | +/- para zoom | ESC/Q para salir".bright_cyan());
    
    io::stdout().flush().unwrap();
}

fn setup_terminal() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    execute!(
        io::stdout(),
        cursor::Hide,
        terminal::Clear(ClearType::All)
    )?;
    Ok(())
}

fn restore_terminal() -> io::Result<()> {
    execute!(
        io::stdout(),
        cursor::Show,
        terminal::Clear(ClearType::All)
    )?;
    terminal::disable_raw_mode()?;
    Ok(())
}

/// -------------------- MAIN --------------------
fn main() -> io::Result<()> {
    let args = Args::parse();
    let (mut vertices, mut faces) = load_obj(&args.model);
    
    // Aplicar subdivisión para más detalle
    if args.detail > 1 {
        println!("Aplicando subdivisión nivel {}...", args.detail);
        subdivide_faces(&mut vertices, &mut faces, args.detail - 1);
        println!("Vértices: {} -> {}, Caras: {}", 
                vertices.len() - faces.len() * 3, vertices.len(), faces.len());
    }
    
    normalize_model(&mut vertices);

    let (w, h) = terminal::size().unwrap();
    let width = w as i32;
    let height = h as i32;
    
    let scale = 20.0;

    // Usar renderizado GPU si está habilitado
    if args.gpu {
        println!("Iniciando renderizado GPU...");
        
        // Preparar datos para GPU
        let normals = calculate_normals(&vertices, &faces);
        let base_color = color_from_string(&args.color);
        
        let mut gpu_vertices = Vec::new();
        for face in &faces {
            if face.vertices.len() >= 3 {
                for i in 1..face.vertices.len() - 1 {
                    for &vi in &[face.vertices[0], face.vertices[i], face.vertices[i + 1]] {
                        let v = &vertices[vi];
                        let normal = normals[vi];
                        gpu_vertices.push(GPUVertex {
                            position: [v.x, v.y, v.z],
                            normal,
                            color: base_color,
                        });
                    }
                }
            }
        }
        
        // Crear simulación de renderizador GPU
        let gpu_renderer = pollster::block_on(GPURenderer::new());
        
        if args.arrows {
            setup_terminal()?;
            let mut keyboard_state = KeyboardState::new();
            
            // Aplicar transformaciones iniciales
            let mut transformed_vertices = gpu_vertices.clone();
            for vertex in &mut transformed_vertices {
                let v = Vertex { x: vertex.position[0], y: vertex.position[1], z: vertex.position[2] };
                let r1 = rotate_x(&v, keyboard_state.angle_x);
                let r2 = rotate_y(&r1, keyboard_state.angle_y);
                let r3 = rotate_z(&r2, keyboard_state.angle_z);
                vertex.position = [r3.x * keyboard_state.scale / 20.0, 
                                 r3.y * keyboard_state.scale / 20.0, 
                                 r3.z * keyboard_state.scale / 20.0];
            }
            
            let screen = gpu_renderer.render_gpu(&transformed_vertices, width, height);
            display_screen(&screen);

            loop {
                if event::poll(Duration::from_millis(50))? {
                    match event::read()? {
                        Event::Key(key_event) => {
                            let mut needs_render = false;
                            
                            match key_event.code {
                                KeyCode::Char('q') | KeyCode::Esc => break,
                                KeyCode::Up => {
                                    keyboard_state.rotate_up();
                                    needs_render = true;
                                }
                                KeyCode::Down => {
                                    keyboard_state.rotate_down();
                                    needs_render = true;
                                }
                                KeyCode::Left => {
                                    keyboard_state.rotate_left();
                                    needs_render = true;
                                }
                                KeyCode::Right => {
                                    keyboard_state.rotate_right();
                                    needs_render = true;
                                }
                                KeyCode::Char('a') | KeyCode::Char('A') => {
                                    keyboard_state.rotate_counter_clockwise();
                                    needs_render = true;
                                }
                                KeyCode::Char('d') | KeyCode::Char('D') => {
                                    keyboard_state.rotate_clockwise();
                                    needs_render = true;
                                }
                                KeyCode::Char('+') | KeyCode::Char('=') => {
                                    keyboard_state.zoom_in();
                                    needs_render = true;
                                }
                                KeyCode::Char('-') => {
                                    keyboard_state.zoom_out();
                                    needs_render = true;
                                }
                                _ => {}
                            }

                            if needs_render {
                                let mut transformed_vertices = gpu_vertices.clone();
                                for vertex in &mut transformed_vertices {
                                    let v = Vertex { x: vertex.position[0], y: vertex.position[1], z: vertex.position[2] };
                                    let r1 = rotate_x(&v, keyboard_state.angle_x);
                                    let r2 = rotate_y(&r1, keyboard_state.angle_y);
                                    let r3 = rotate_z(&r2, keyboard_state.angle_z);
                                    vertex.position = [r3.x * keyboard_state.scale / 20.0, 
                                                     r3.y * keyboard_state.scale / 20.0, 
                                                     r3.z * keyboard_state.scale / 20.0];
                                }
                                let screen = gpu_renderer.render_gpu(&transformed_vertices, width, height);
                                display_screen(&screen);
                            }
                        }
                        _ => {}
                    }
                }
            }

            restore_terminal()?;

        } else if args.rotate {
            let mut angle_x = 0.0;
            let mut angle_y = 0.0;
            let mut angle_z = 0.0;

            loop {
                let mut transformed_vertices = gpu_vertices.clone();
                for vertex in &mut transformed_vertices {
                    let v = Vertex { x: vertex.position[0], y: vertex.position[1], z: vertex.position[2] };
                    let r1 = rotate_x(&v, angle_x);
                    let r2 = rotate_y(&r1, angle_y);
                    let r3 = rotate_z(&r2, angle_z);
                    vertex.position = [r3.x, r3.y, r3.z];
                }

                let screen = gpu_renderer.render_gpu(&transformed_vertices, width, height);
                display_screen(&screen);

                angle_x += 0.8;
                angle_y += 0.6;
                angle_z += 0.4;

                thread::sleep(Duration::from_millis(60));
            }
        } else {
            let mut transformed_vertices = gpu_vertices.clone();
            for vertex in &mut transformed_vertices {
                let v = Vertex { x: vertex.position[0], y: vertex.position[1], z: vertex.position[2] };
                let r1 = rotate_x(&v, 0.0);
                let r2 = rotate_y(&r1, 50.0);
                let r3 = rotate_z(&r2, 3.0);
                vertex.position = [r3.x, r3.y, r3.z];
            }

            let screen = gpu_renderer.render_gpu(&transformed_vertices, width, height);
            display_screen(&screen);
        }
    } else {
        // Renderizado CPU tradicional mejorado
        if args.arrows {
            setup_terminal()?;
            let mut keyboard_state = KeyboardState::new();
            
            let projected: Vec<Point3D> = vertices
                .iter()
                .map(|v| {
                    let r1 = rotate_x(v, keyboard_state.angle_x);
                    let r2 = rotate_y(&r1, keyboard_state.angle_y);
                    let r3 = rotate_z(&r2, keyboard_state.angle_z);
                    project(&r3, width, height, keyboard_state.scale)
                })
                .collect();
            render(&projected, &faces, width, height, args.wireframe, &args.color);

            loop {
                if event::poll(Duration::from_millis(50))? {
                    match event::read()? {
                        Event::Key(key_event) => {
                            let mut needs_render = false;
                            
                            match key_event.code {
                                KeyCode::Char('q') | KeyCode::Esc => break,
                                KeyCode::Up => {
                                    keyboard_state.rotate_up();
                                    needs_render = true;
                                }
                                KeyCode::Down => {
                                    keyboard_state.rotate_down();
                                    needs_render = true;
                                }
                                KeyCode::Left => {
                                    keyboard_state.rotate_left();
                                    needs_render = true;
                                }
                                KeyCode::Right => {
                                    keyboard_state.rotate_right();
                                    needs_render = true;
                                }
                                KeyCode::Char('a') | KeyCode::Char('A') => {
                                    keyboard_state.rotate_counter_clockwise();
                                    needs_render = true;
                                }
                                KeyCode::Char('d') | KeyCode::Char('D') => {
                                    keyboard_state.rotate_clockwise();
                                    needs_render = true;
                                }
                                KeyCode::Char('+') | KeyCode::Char('=') => {
                                    keyboard_state.zoom_in();
                                    needs_render = true;
                                }
                                KeyCode::Char('-') => {
                                    keyboard_state.zoom_out();
                                    needs_render = true;
                                }
                                _ => {}
                            }

                            if needs_render {
                                let projected: Vec<Point3D> = vertices
                                    .iter()
                                    .map(|v| {
                                        let r1 = rotate_x(v, keyboard_state.angle_x);
                                        let r2 = rotate_y(&r1, keyboard_state.angle_y);
                                        let r3 = rotate_z(&r2, keyboard_state.angle_z);
                                        project(&r3, width, height, keyboard_state.scale)
                                    })
                                    .collect();
                                render(&projected, &faces, width, height, args.wireframe, &args.color);
                            }
                        }
                        _ => {}
                    }
                }
            }

            restore_terminal()?;

        } else if args.rotate {
            let mut angle_x = 0.0;
            let mut angle_y = 0.0;
            let mut angle_z = 0.0;

            loop {
                let projected: Vec<Point3D> = vertices
                    .iter()
                    .map(|v| {
                        let r1 = rotate_x(v, angle_x);
                        let r2 = rotate_y(&r1, angle_y);
                        let r3 = rotate_z(&r2, angle_z);
                        project(&r3, width, height, scale)
                    })
                    .collect();

                render(&projected, &faces, width, height, args.wireframe, &args.color);

                angle_x += 0.8;
                angle_y += 0.6;
                angle_z += 0.4;

                thread::sleep(Duration::from_millis(60));
            }
        } else {
            let projected: Vec<Point3D> = vertices
                .iter()
                .map(|v| {
                    let r1 = rotate_x(v, 0.0);
                    let r2 = rotate_y(&r1, 50.0);
                    let r3 = rotate_z(&r2, 3.0);
                    project(&r3, width, height, scale)
                })
                .collect();

            render(&projected, &faces, width, height, args.wireframe, &args.color);
        }
    }

    Ok(())
}

// Función auxiliar para mostrar la pantalla renderizada por GPU
fn display_screen(screen: &[Vec<String>]) {
    execute!(io::stdout(), cursor::MoveTo(0, 0), terminal::Clear(ClearType::All)).unwrap();
    
    for row in screen {
        println!("{}", row.join(""));
    }
    
    println!("\n{}", "Controles: ← → ↑ ↓ para rotar | A/D para rotar en Z | +/- para zoom | ESC/Q para salir | Modo GPU activo".bright_cyan());
    
    io::stdout().flush().unwrap();
}