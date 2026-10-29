//! Relleno por líneas horizontales (scanline).
//!
//! Avanza una fila a la vez. Para cada fila toma la recta horizontal que pasa
//! por los centros de sus píxeles, `y + 0.5`, busca dónde la cortan las aristas
//! del triángulo y genera fragmentos entre los dos cortes.
//!
//! Regla de bordes, equivalente a la "top-left rule" de `barycentric.rs`:
//! - Una arista cubre las filas cuyo centro cumple `y_min <= y + 0.5 < y_max`
//!   (intervalo semiabierto). Un vértice que cae justo sobre el centro de una
//!   fila lo cuenta solo la arista que empieza ahí, no la que termina.
//! - Las aristas horizontales no cortan ninguna fila y se ignoran; la fila de
//!   una arista horizontal superior sí entra (es el `y_min` de las otras dos) y
//!   la de una inferior no (es su `y_max`).
//! - En cada fila se aceptan los centros con `x_left <= x + 0.5 < x_right`: el
//!   borde izquierdo entra, el derecho no.

use crate::barycentric::edge_function;
use crate::fragment::Fragment;
use crate::vertex::Vertex;

// Sin inline para que aparezca con su nombre en el flamegraph.
#[inline(never)]
pub fn triangle_scanline(v1: &Vertex, v2: &Vertex, v3: &Vertex) -> Vec<Fragment> {
    let a = v1.transformed_position;
    let mut b = v2.transformed_position;
    let mut c = v3.transformed_position;

    let area = edge_function(a, b, c.x, c.y);

    // Mismo caso degenerado que en el algoritmo baricéntrico.
    if area == 0.0 || !area.is_finite() {
        return Vec::new();
    }

    // Con área positiva y `y` hacia abajo, las aristas que suben (dy < 0) son
    // el borde izquierdo y las que bajan (dy > 0) el derecho.
    if area < 0.0 {
        std::mem::swap(&mut b, &mut c);
    }

    let color = v1.color;
    let depth = (a.z + b.z + c.z) / 3.0;

    // Para cada arista no horizontal: punto de inicio, pendiente dx/dy,
    // rango de y que cubre y si es izquierda o derecha.
    let edges = [(a, b), (b, c), (c, a)];
    let mut active = Vec::with_capacity(3);
    for (p, q) in edges {
        let dy = q.y - p.y;
        if dy == 0.0 {
            continue;
        }
        active.push((p, (q.x - p.x) / dy, p.y.min(q.y), p.y.max(q.y), dy < 0.0));
    }

    let min_y = a.y.min(b.y).min(c.y);
    let max_y = a.y.max(b.y).max(c.y);

    // Filas cuyo centro y + 0.5 está en [min_y, max_y).
    let first_row = (min_y - 0.5).ceil() as i32;
    let end_row = (max_y - 0.5).ceil() as i32;

    let mut fragments = Vec::new();

    for y in first_row..end_row {
        let py = y as f32 + 0.5;

        let mut x_left = f32::INFINITY;
        let mut x_right = f32::NEG_INFINITY;

        for &(p, dxdy, y_min, y_max, is_left) in &active {
            if py < y_min || py >= y_max {
                continue;
            }
            // Se evalúa la recta en cada fila en vez de sumar dxdy fila tras
            // fila: sumar acumula error de redondeo y movería los bordes.
            let x = p.x + (py - p.y) * dxdy;
            if is_left {
                x_left = x;
            } else {
                x_right = x;
            }
        }

        if !(x_left < x_right) {
            continue;
        }

        // Primer x con x + 0.5 >= x_left y último con x + 0.5 < x_right.
        let first_x = (x_left - 0.5).ceil() as i32;
        let end_x = (x_right - 0.5).ceil() as i32;

        for x in first_x..end_x {
            fragments.push(Fragment::new(x as f32, y as f32, color, depth));
        }
    }

    fragments
}
