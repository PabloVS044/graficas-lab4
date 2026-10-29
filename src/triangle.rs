use crate::fragment::Fragment;
use crate::vertex::Vertex;
use crate::line::line;
use crate::barycentric::triangle_barycentric;
use crate::scanline::triangle_scanline;

#[derive(Clone, Copy)]
pub enum TriangleMode {
    Wireframe,
    FilledBarycentric,
    FilledScanline,
}

pub fn triangle(v1: &Vertex, v2: &Vertex, v3: &Vertex, mode: TriangleMode) -> Vec<Fragment> {
    match mode {
        TriangleMode::Wireframe => triangle_wireframe(v1, v2, v3),
        TriangleMode::FilledBarycentric => triangle_barycentric(v1, v2, v3),
        TriangleMode::FilledScanline => triangle_scanline(v1, v2, v3),
    }
}

pub fn triangle_wireframe(v1: &Vertex, v2: &Vertex, v3: &Vertex) -> Vec<Fragment> {
    let mut fragments = Vec::new();

    // Draw the three sides of the triangle
    fragments.extend(line(v1, v2));
    fragments.extend(line(v2, v3));
    fragments.extend(line(v3, v1));

    fragments
}
