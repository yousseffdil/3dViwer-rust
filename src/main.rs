use clap::Parser;
use colored::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::f32::consts::PI;
use std::{thread, time::Duration};

/// Herramienta CLI para mostrar wireframes de modelos 3D en el terminal
#[derive(Parser, Debug)]
#[command(name = "showObj", version, about = "Muestra modelos .obj en terminal")]
struct Args {
    #[arg(short = 'm', long = "model")]
    model: String,

    #[arg(long, default_value_t = false)]
    rotate: bool,
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

/// -------------------- ROTACIONES --------------------
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

/// -------------------- NORMALIZACIÓN --------------------
fn normalize_model(vertices: &mut Vec<Vertex>) {
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    let mut min_z = f32::MAX;
    let mut max_z = f32::MIN;

    // calcular bounding box
    for v in vertices.iter() {
        if v.x < min_x { min_x = v.x; }
        if v.x > max_x { max_x = v.x; }
        if v.y < min_y { min_y = v.y; }
        if v.y > max_y { max_y = v.y; }
        if v.z < min_z { min_z = v.z; }
        if v.z > max_z { max_z = v.z; }
    }

    // centro del modelo
    let center_x = (min_x + max_x) / 1.5;
    let center_y = (min_y + max_y) / 1.5;
    let center_z = (min_z + max_z) / 1.5;

    // mayor dimensión para escalar
    let size_x = max_x - min_x;
    let size_y = max_y - min_y;
    let size_z = max_z - min_z;
    let max_size = size_x.max(size_y).max(size_z);

    // trasladar y escalar
    for v in vertices.iter_mut() {
        v.x = (v.x - center_x) / max_size;
        v.y = (v.y - center_y) / max_size;
        v.z = (v.z - center_z) / max_size;
    }
}

/// -------------------- CARGAR OBJ --------------------
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

/// -------------------- GRADIENTE CON COLORES --------------------
fn get_gradient_char_with_color(z: f32) -> String {
    match z {
        z if z > 0.3 => "█".red().to_string(),      // Muy cerca - Blanco brillante
        z if z > 0.1 => "▓".yellow().to_string(),     // Cerca - Amarillo brillante  
        z if z > -0.1 => "▒".green().to_string(),            // Medio - Verde
        z if z > -0.3 => "░".blue().to_string(),             // Lejos - Azul
        _ => "|".magenta().to_string()                       // Muy lejos - Magenta
    }
}

// Interpolar entre dos valores z para obtener el carácter correcto
fn lerp_z(z1: f32, z2: f32, t: f32) -> f32 {
    z1 + (z2 - z1) * t
}

/// -------------------- DIBUJO CON COLORES --------------------
fn draw_line_gradient(screen: &mut Vec<Vec<String>>, p1: Point3D, p2: Point3D) {
    let mut x0 = p1.x;
    let mut y0 = p1.y;
    let z0 = p1.z;
    let x1 = p2.x;
    let y1 = p2.y;
    let z1 = p2.z;

    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    let total_distance = ((dx * dx + dy * dy) as f32).sqrt();

    loop {
        if y0 >= 0 && y0 < screen.len() as i32 && x0 >= 0 && x0 < screen[0].len() as i32 {
            let current_distance = (((x0 - p1.x) * (x0 - p1.x) + (y0 - p1.y) * (y0 - p1.y)) as f32).sqrt();
            let t = if total_distance > 0.0 { current_distance / total_distance } else { 0.0 };
            
            let current_z = lerp_z(z0, z1, t);
            
            screen[y0 as usize][x0 as usize] = get_gradient_char_with_color(current_z);
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

fn render_wireframe(points: &[Point3D], faces: &[Face], width: i32, height: i32) {
    let mut screen = vec![vec![" ".to_string(); width as usize]; height as usize];

    for face in faces {
        for i in 0..face.vertices.len() {
            let v1 = points[face.vertices[i]];
            let v2 = points[face.vertices[(i + 1) % face.vertices.len()]];
            draw_line_gradient(&mut screen, v1, v2);
        }
    }

    print!("\x1B[2J\x1B[H");
    

    for row in screen {
        let line: String = row.into_iter().collect();
        println!("{}", line);
    }
}

/// -------------------- MAIN --------------------
fn main() {
    let args = Args::parse();
    let (mut vertices, faces) = load_obj(&args.model);

    normalize_model(&mut vertices);

    let width = 120;  
    let height = 60; 
    let scale = 20.0; 
    
    //let mut scale = 20.0; 

    let mut angle_x = 0.0;
    let mut angle_y = 0.0;
    let mut angle_z = 0.0;

    if args.rotate {
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
    
            render_wireframe(&projected, &faces, width, height);
    
            angle_x += 0.8;
            angle_y += 0.6;
            angle_z += 0.4;
    
            // scale += 0.10;
            // if scale > 40.0 {
            //     scale = 20.0;
            // }

            thread::sleep(Duration::from_millis(16));
        }
    }else{
        let projected: Vec<Point3D> = vertices
                .iter()
                .map(|v| {
                    let r1 = rotate_x(v, 0.0);
                    let r2 = rotate_y(&r1, 50.0);
                    let r3 = rotate_z(&r2, 3.0);
                    project(&r3, width, height, scale)
                })
                .collect();
    
            render_wireframe(&projected, &faces, width, height);
    

    }
   
}