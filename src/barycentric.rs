//! Relleno con coordenadas baricéntricas.
//!
//! Recorre el rectángulo más pequeño que contiene al triángulo y evalúa el
//! centro de cada píxel, `(x + 0.5, y + 0.5)`. Un centro es parte del triángulo
//! si sus tres coordenadas baricéntricas lo ponen dentro.
//!
//! Regla de bordes (la misma que sigue `scanline.rs`): un centro que cae
//! exactamente sobre una arista solo se acepta si esa arista es **superior o
//! izquierda** ("top-left rule"). Así dos triángulos que comparten una arista
//! no generan el mismo fragmento dos veces ni dejan un hueco entre ellos.

use raylib::math::Vector3;

use crate::fragment::Fragment;
use crate::vertex::Vertex;

/// Función de arista: el doble del área con signo del triángulo `a, b, p`.
/// Es positiva cuando `p` queda del lado interior de la arista `a -> b`
/// (con los vértices ya ordenados para que el área total sea positiva).
#[inline]
pub fn edge_function(a: Vector3, b: Vector3, px: f32, py: f32) -> f32 {
    (b.x - a.x) * (py - a.y) - (b.y - a.y) * (px - a.x)
}

/// Con `y` creciendo hacia abajo y el área positiva, una arista es:
/// - superior si es horizontal y el interior queda debajo (`dy == 0`, `dx > 0`);
/// - izquierda si el interior queda a su derecha (`dy < 0`).
#[inline]
fn is_top_left(a: Vector3, b: Vector3) -> bool {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    (dy == 0.0 && dx > 0.0) || dy < 0.0
}

/// Una coordenada baricéntrica acepta el punto si es positiva, o si es cero y
/// la arista correspondiente es superior o izquierda.
#[inline]
fn inside(weight: f32, top_left: bool) -> bool {
    weight > 0.0 || (weight == 0.0 && top_left)
}

// Sin inline para que aparezca con su nombre en el flamegraph.
#[inline(never)]
pub fn triangle_barycentric(v1: &Vertex, v2: &Vertex, v3: &Vertex) -> Vec<Fragment> {
    let a = v1.transformed_position;
    let mut b = v2.transformed_position;
    let mut c = v3.transformed_position;

    let mut area = edge_function(a, b, c.x, c.y);

    // Tres vértices sobre una misma línea: no hay interior y no se puede
    // dividir entre el área.
    if area == 0.0 || !area.is_finite() {
        return Vec::new();
    }

    // Si el orden viene invertido el área sale negativa. Intercambiar b y c lo
    // corrige sin cambiar el triángulo, y la regla de bordes sigue siendo la
    // misma para ambos órdenes.
    if area < 0.0 {
        std::mem::swap(&mut b, &mut c);
        area = -area;
    }

    let top_left_bc = is_top_left(b, c);
    let top_left_ca = is_top_left(c, a);
    let top_left_ab = is_top_left(a, b);

    // Rectángulo envolvente. Cubre todos los centros x + 0.5 dentro de
    // [min_x, max_x]; la columna extra que pueda aparecer la descarta la prueba.
    let min_x = a.x.min(b.x).min(c.x).floor() as i32;
    let max_x = a.x.max(b.x).max(c.x).ceil() as i32;
    let min_y = a.y.min(b.y).min(c.y).floor() as i32;
    let max_y = a.y.max(b.y).max(c.y).ceil() as i32;

    let color = v1.color;
    let depth = (a.z + b.z + c.z) / 3.0;
    let inv_area = 1.0 / area;

    let mut fragments = Vec::new();

    for y in min_y..max_y {
        let py = y as f32 + 0.5;
        for x in min_x..max_x {
            let px = x as f32 + 0.5;

            // Coordenadas baricéntricas: cada peso es el área del subtriángulo
            // opuesto a un vértice dividida entre el área total.
            let w1 = edge_function(b, c, px, py) * inv_area;
            let w2 = edge_function(c, a, px, py) * inv_area;
            let w3 = edge_function(a, b, px, py) * inv_area;

            if inside(w1, top_left_bc) && inside(w2, top_left_ca) && inside(w3, top_left_ab) {
                fragments.push(Fragment::new(x as f32, y as f32, color, depth));
            }
        }
    }

    fragments
}
