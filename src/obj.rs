//! Cargador de archivos OBJ.
//!
//! Solo lee lo que necesita el renderer:
//! - `v x y z`: posición de un vértice;
//! - `vn x y z`: normal;
//! - `vt u v`: coordenada de textura;
//! - `f a b c ...`: una cara, con índices `v`, `v/vt`, `v//vn` o `v/vt/vn`.
//!
//! Las demás líneas (`o`, `s`, `l`, `usemtl`, `mtllib`, comentarios...) se
//! ignoran. Una cara con más de tres vértices se divide en triángulos en
//! abanico desde su primer vértice.

use raylib::math::{Vector2, Vector3};

use crate::vertex::Vertex;

/// Índices (desde 0) de un vértice de una cara. `vt` y `vn` son opcionales.
#[derive(Clone, Copy, Debug)]
pub struct FaceIndex {
    pub position: usize,
    pub tex_coords: Option<usize>,
    pub normal: Option<usize>,
}

pub struct Obj {
    pub vertices: Vec<Vector3>,
    pub normals: Vec<Vector3>,
    pub tex_coords: Vec<Vector2>,
    /// Cada triángulo son tres índices.
    pub triangles: Vec<[FaceIndex; 3]>,
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
            normals: Vec::new(),
            tex_coords: Vec::new(),
            triangles: Vec::new(),
        };

        for (number, line) in text.lines().enumerate() {
            let number = number + 1;
            let mut parts = line.split_whitespace();

            match parts.next() {
                Some("v") => {
                    let [x, y, z] = parse_floats(&mut parts, number)?;
                    obj.vertices.push(Vector3::new(x, y, z));
                }
                Some("vn") => {
                    let [x, y, z] = parse_floats(&mut parts, number)?;
                    obj.normals.push(Vector3::new(x, y, z));
                }
                Some("vt") => {
                    let [u, v] = parse_floats(&mut parts, number)?;
                    obj.tex_coords.push(Vector2::new(u, v));
                }
                Some("f") => {
                    let face = parts
                        .map(|part| obj.parse_face_index(part, number))
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

    /// Lee un vértice de una cara, por ejemplo `12/4/7`, `12//7` o `12`.
    fn parse_face_index(&self, part: &str, number: usize) -> Result<FaceIndex, String> {
        let mut fields = part.split('/');

        let position = fields.next().unwrap_or("");
        let tex_coords = fields.next().filter(|field| !field.is_empty());
        let normal = fields.next().filter(|field| !field.is_empty());

        Ok(FaceIndex {
            position: resolve_index(position, self.vertices.len(), number)?,
            tex_coords: tex_coords
                .map(|field| resolve_index(field, self.tex_coords.len(), number))
                .transpose()?,
            normal: normal
                .map(|field| resolve_index(field, self.normals.len(), number))
                .transpose()?,
        })
    }

    /// Recorre los triángulos y, con sus tres índices, busca los tres vértices
    /// de cada uno. El resultado tiene 3 vértices por triángulo, en el formato
    /// que espera `primitive_assembly`.
    pub fn vertex_array(&self, color: Vector3) -> Vec<Vertex> {
        let mut vertex_array = Vec::with_capacity(self.triangles.len() * 3);

        for triangle in &self.triangles {
            for index in triangle {
                let position = self.vertices[index.position];
                let normal = index
                    .normal
                    .map_or(Vector3::new(0.0, 0.0, 0.0), |i| self.normals[i]);
                let tex_coords = index
                    .tex_coords
                    .map_or(Vector2::new(0.0, 0.0), |i| self.tex_coords[i]);

                let mut vertex = Vertex::new(position, normal, tex_coords);
                vertex.color = color;
                vertex_array.push(vertex);
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

fn parse_floats<'a, const N: usize>(
    parts: &mut impl Iterator<Item = &'a str>,
    number: usize,
) -> Result<[f32; N], String> {
    let mut values = [0.0; N];

    for value in values.iter_mut() {
        let part = parts
            .next()
            .ok_or_else(|| format!("línea {number}: faltan componentes"))?;
        *value = part
            .parse()
            .map_err(|_| format!("línea {number}: \"{part}\" no es un número"))?;
    }

    Ok(values)
}

/// Los índices del OBJ empiezan en 1. Un índice negativo cuenta desde el
/// último elemento leído hasta ese momento (-1 es el último).
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

        assert_eq!(obj.triangles.len(), 2);
        let positions: Vec<[usize; 3]> = obj
            .triangles
            .iter()
            .map(|t| [t[0].position, t[1].position, t[2].position])
            .collect();
        assert_eq!(positions, vec![[0, 1, 2], [0, 2, 3]]);
        assert_eq!(obj.triangles[0][0].normal, Some(0));
        assert_eq!(obj.triangles[0][0].tex_coords, None);
    }

    #[test]
    fn resolves_negative_indices() {
        let obj = Obj::parse("v 0 0 0\nv 1 0 0\nv 0 1 0\nf -3 -2 -1\n").unwrap();
        let t = obj.triangles[0];
        assert_eq!([t[0].position, t[1].position, t[2].position], [0, 1, 2]);
    }

    #[test]
    fn rejects_out_of_range_indices() {
        assert!(Obj::parse("v 0 0 0\nf 1 2 3\n").is_err());
    }
}
