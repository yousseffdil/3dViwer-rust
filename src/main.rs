use clap::Parser;

/// Herramienta CLI para mostrar wireframes de modelos 3D en el terminal
#[derive(Parser, Debug)]
#[command(name = "showObj", version, about = "Muestra modelos .obj en terminal")]
struct Args {
    #[arg(short = 'm', long = "model")]
    model: String,

    #[arg(long, default_value_t = false)]
    rotate: bool,
}

fn main() {
    let args = Args::parse();

    println!("Modelo: {}", args.model);
    println!("Rotación automática: {}", args.rotate);
}
