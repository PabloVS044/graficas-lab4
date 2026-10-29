// main.rs

mod framebuffer;
mod triangle;
mod line;
mod vertex;
mod fragment;
mod shaders;
mod primitive;
mod rasterizer;
mod obj;

use framebuffer::Framebuffer;
use obj::Obj;
use vertex::Vertex;
use shaders::{vertex_shader, fragment_shader};
use primitive::primitive_assembly;
use rasterizer::rasterize;
use raylib::prelude::*;

pub struct Uniforms {
    pub model_matrix: Matrix,
}

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const MODEL_PATH: &str = "assets/nave_graficas.obj";

/// Matriz que escala y traslada el modelo para verlo desde arriba: la `x`
/// del OBJ queda como `x` de pantalla y la `z` como `y`. Con la vista
/// frontal (`x`, `y`) la nave se veía solo como una silueta.
///
/// El modelo queda centrado y ocupa como máximo el 80% de la ventana. La `z`
/// se invierte para que la punta de la nave apunte hacia arriba.
fn fit_to_screen(min: Vector3, max: Vector3, width: f32, height: f32) -> Matrix {
    let size_x = (max.x - min.x).max(f32::EPSILON);
    let size_z = (max.z - min.z).max(f32::EPSILON);
    let scale = 0.8 * (width / size_x).min(height / size_z);

    let center_x = (min.x + max.x) / 2.0;
    let center_z = (min.z + max.z) / 2.0;

    // x' =  scale * (x - center_x) + width / 2
    // y' = -scale * (z - center_z) + height / 2
    // z' =  scale * y
    let mut matrix = Matrix::identity();
    matrix.m0 = scale;
    matrix.m5 = 0.0;
    matrix.m9 = -scale;
    matrix.m6 = scale;
    matrix.m10 = 0.0;
    matrix.m12 = width / 2.0 - scale * center_x;
    matrix.m13 = height / 2.0 + scale * center_z;
    matrix
}

fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[Vertex],
) {
    // 1. Procesamiento de vértices
    let transformed_vertices: Vec<Vertex> = vertex_array
        .iter()
        .map(|vertex| vertex_shader(vertex, uniforms))
        .collect();

    // 2. Ensamblaje de primitivas
    let triangles = primitive_assembly(&transformed_vertices);

    // 3. Rasterización
    let fragments = rasterize(&triangles);

    // 4. Procesamiento de fragmentos
    for fragment in fragments {
        let color = fragment_shader(&fragment);

        framebuffer.point(
            fragment.position.x as i32,
            fragment.position.y as i32,
            color,
        );
    }
}

fn main() {
    let model = Obj::load(MODEL_PATH).unwrap_or_else(|err| panic!("{err}"));
    let vertex_array = model.vertex_array();
    let (min, max) = model.bounds();
    let uniforms = Uniforms {
        model_matrix: fit_to_screen(min, max, WIDTH as f32, HEIGHT as f32),
    };

    if std::env::args().any(|arg| arg == "--export-image") {
        export_image(&uniforms, &vertex_array);
        return;
    }

    let (mut window, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Lab 4: Carga de modelos")
        .build();

    window.set_target_fps(60);

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    framebuffer.set_background_color(Vector3::new(0.0, 0.0, 0.0));
    framebuffer.init_texture(&mut window, &thread);

    while !window.window_should_close() {
        framebuffer.clear();
        render(&mut framebuffer, &uniforms, &vertex_array);
        framebuffer.swap_buffers(&mut window, &thread);
    }
}

/// Guarda el modelo dibujado en `docs/modelo.png`, sin abrir la ventana.
fn export_image(uniforms: &Uniforms, vertex_array: &[Vertex]) {
    std::fs::create_dir_all("docs").expect("No se pudo crear docs/");

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    framebuffer.set_background_color(Vector3::new(0.0, 0.0, 0.0));
    framebuffer.clear();

    render(&mut framebuffer, uniforms, vertex_array);
    framebuffer.export("docs/modelo.png");
}
