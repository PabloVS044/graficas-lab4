// main.rs

mod framebuffer;
mod triangle;
mod line;
mod vertex;
mod fragment;
mod shaders;
mod primitive;
mod rasterizer;
mod barycentric;
mod scanline;
mod obj;

use framebuffer::Framebuffer;
use obj::Obj;
use vertex::Vertex;
use triangle::TriangleMode;
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

/// Matriz que escala y traslada el modelo para que su `x` y su `y` queden
/// centradas en la pantalla y ocupen como máximo el 80% de la ventana.
/// La `y` se invierte: en el OBJ crece hacia arriba y en pantalla hacia abajo.
fn fit_to_screen(min: Vector3, max: Vector3, width: f32, height: f32) -> Matrix {
    let size_x = (max.x - min.x).max(f32::EPSILON);
    let size_y = (max.y - min.y).max(f32::EPSILON);
    let scale = 0.8 * (width / size_x).min(height / size_y);

    let center_x = (min.x + max.x) / 2.0;
    let center_y = (min.y + max.y) / 2.0;

    // x' = scale * (x - center_x) + width / 2
    // y' = -scale * (y - center_y) + height / 2
    let mut matrix = Matrix::identity();
    matrix.m0 = scale;
    matrix.m5 = -scale;
    matrix.m10 = scale;
    matrix.m12 = width / 2.0 - scale * center_x;
    matrix.m13 = height / 2.0 + scale * center_y;
    matrix
}

fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[Vertex],
    mode: TriangleMode,
) {
    // 1. Procesamiento de vértices
    let transformed_vertices: Vec<Vertex> = vertex_array
        .iter()
        .map(|vertex| vertex_shader(vertex, uniforms))
        .collect();

    // 2. Ensamblaje de primitivas
    let triangles = primitive_assembly(&transformed_vertices);

    // 3. Rasterización
    let fragments = rasterize(&triangles, mode);

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
    let vertex_array = model.vertex_array(Vector3::new(1.0, 1.0, 1.0));
    let (min, max) = model.bounds();
    let uniforms = Uniforms {
        model_matrix: fit_to_screen(min, max, WIDTH as f32, HEIGHT as f32),
    };

    let (mut window, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Lab 4: Carga de modelos")
        .build();

    window.set_target_fps(60);

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    framebuffer.set_background_color(Vector3::new(0.0, 0.0, 0.0));
    framebuffer.init_texture(&mut window, &thread);

    let mut mode = TriangleMode::Wireframe;

    while !window.window_should_close() {
        if window.is_key_pressed(KeyboardKey::KEY_ONE) {
            mode = TriangleMode::Wireframe;
        }
        if window.is_key_pressed(KeyboardKey::KEY_TWO) {
            mode = TriangleMode::FilledBarycentric;
        }
        if window.is_key_pressed(KeyboardKey::KEY_THREE) {
            mode = TriangleMode::FilledScanline;
        }

        framebuffer.clear();
        render(&mut framebuffer, &uniforms, &vertex_array, mode);
        framebuffer.swap_buffers(&mut window, &thread);
    }
}
