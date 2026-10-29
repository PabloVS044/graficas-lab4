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
use vertex::Vertex;
use triangle::TriangleMode;
use shaders::{vertex_shader, fragment_shader};
use primitive::primitive_assembly;
use rasterizer::rasterize;
use raylib::prelude::*;

pub struct Uniforms {
    pub model_matrix: Matrix,
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
    let (mut window, thread) = raylib::init()
        .size(800, 600)
        .title("Lab 4: Carga de modelos")
        .build();

    window.set_target_fps(60);

    let mut framebuffer = Framebuffer::new(800, 600);
    framebuffer.set_background_color(Vector3::new(0.0, 0.0, 0.0));
    framebuffer.init_texture(&mut window, &thread);

    // Triángulo de prueba en coordenadas de pantalla.
    let color = Vector3::new(1.0, 1.0, 1.0);
    let vertex_array = vec![
        Vertex::new_with_color(Vector3::new(400.0, 150.0, 0.0), color),
        Vertex::new_with_color(Vector3::new(200.0, 450.0, 0.0), color),
        Vertex::new_with_color(Vector3::new(600.0, 450.0, 0.0), color),
    ];
    let uniforms = Uniforms {
        model_matrix: Matrix::identity(),
    };
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
