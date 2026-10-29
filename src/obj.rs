//! Cargador de archivos OBJ.
//!
//! Solo lee lo que necesita el renderer:
//! - `v x y z`: posición de un vértice;
//! - `f a b c ...`: una cara. Cada vértice puede venir como `v`, `v/vt`,
//!   `v//vn` o `v/vt/vn`; solo se usa el índice de la posición.
//!
//! Las demás líneas (`vn`, `vt`, `o`, `s`, `l`, `usemtl`, comentarios...) se
//! ignoran. Una cara con más de tres vértices se divide en triángulos en
//! abanico desde su primer vértice.

use raylib::math::Vector3;

use crate::vertex::Vertex;

pub struct Obj {
    pub vertices: Vec<Vector3>,
    /// Cada triángulo son tres índices (desde 0) de `vertices`.
    pub triangles: Vec<[usize; 3]>,
}

impl Obj {
    pub fn load(path: &str) -> Result<Obj, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|err| format!("No se pudo leer {path}: {err}"))?;

        Obj::parse(&text).map_err(|err| format!("{path}: {err}"))
    }

    pub fn parse(text: &str) -> Result<Obj, String> {
        let mut obj = Obj {
            vertices: Vec::new(),
            triangles: Vec::new(),
        };

        for (number, line) in text.lines().enumerate() {
            let number = number + 1;
            let mut parts = line.split_whitespace();

            match parts.next() {
                Some("v") => {
                    let mut coordinate = || -> Result<f32, String> {
                        let part = parts
                            .next()
                            .ok_or_else(|| format!("línea {number}: faltan componentes"))?;
                        part.parse()
                            .map_err(|_| format!("línea {number}: \"{part}\" no es un número"))
                    };
                    let (x, y, z) = (coordinate()?, coordinate()?, coordinate()?);
                    obj.vertices.push(Vector3::new(x, y, z));
                }
                Some("f") => {
                    let face = parts
                        .map(|part| {
                            // De `12/4/7` solo interesa el `12`.
                            let position = part.split('/').next().unwrap_or("");
                            resolve_index(position, obj.vertices.len(), number)
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    if face.len() < 3 {
                        return Err(format!("línea {number}: una cara necesita al menos 3 vértices"));
                    }

                    // Abanico: (0, 1, 2), (0, 2, 3), ...
                    for i in 1..face.len() - 1 {
                        obj.triangles.push([face[0], face[i], face[i + 1]]);
                    }
                }
                _ => {}
            }
        }

        Ok(obj)
    }

    /// Recorre los triángulos y, con sus tres índices, busca los tres vértices
    /// de cada uno. El resultado tiene 3 vértices por triángulo, en el formato
    /// que espera `primitive_assembly`.
    pub fn vertex_array(&self) -> Vec<Vertex> {
        let mut vertex_array = Vec::with_capacity(self.triangles.len() * 3);

        for triangle in &self.triangles {
            for &index in triangle {
                vertex_array.push(Vertex::new(self.vertices[index]));
            }
        }

        vertex_array
    }

    /// Esquinas mínima y máxima de la caja que contiene a todos los vértices.
    pub fn bounds(&self) -> (Vector3, Vector3) {
        let mut min = Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);

        for v in &self.vertices {
            min = Vector3::new(min.x.min(v.x), min.y.min(v.y), min.z.min(v.z));
            max = Vector3::new(max.x.max(v.x), max.y.max(v.y), max.z.max(v.z));
        }

        (min, max)
    }
}

/// Los índices del OBJ empiezan en 1. Un índice negativo cuenta desde el
/// último vértice leído hasta ese momento (-1 es el último).
fn resolve_index(field: &str, len: usize, number: usize) -> Result<usize, String> {
    let index: i64 = field
        .parse()
        .map_err(|_| format!("línea {number}: \"{field}\" no es un índice"))?;

    let resolved = if index > 0 {
        index - 1
    } else {
        len as i64 + index
    };

    if index == 0 || resolved < 0 || resolved >= len as i64 {
        return Err(format!("línea {number}: el índice {index} está fuera de rango"));
    }

    Ok(resolved as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_quads_into_two_triangles() {
        let obj = Obj::parse(
            "v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nvn 0 0 1\nf 1//1 2//1 3//1 4//1\n",
        )
        .unwrap();

        assert_eq!(obj.triangles, vec![[0, 1, 2], [0, 2, 3]]);
    }

    #[test]
    fn resolves_negative_indices() {
        let obj = Obj::parse("v 0 0 0\nv 1 0 0\nv 0 1 0\nf -3 -2 -1\n").unwrap();
        assert_eq!(obj.triangles, vec![[0, 1, 2]]);
    }

    #[test]
    fn rejects_out_of_range_indices() {
        assert!(Obj::parse("v 0 0 0\nf 1 2 3\n").is_err());
    }
}
