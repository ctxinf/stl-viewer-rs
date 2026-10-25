#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub center: [f32; 3],
    pub radius: f32,
}
impl Mesh {
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        let mut vertices = Vec::new();
        let count = data
            .get(80..84)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()) as usize);
        if let Some(count) =
            count.filter(|n| n.checked_mul(50).and_then(|n| n.checked_add(84)) == Some(data.len()))
        {
            vertices
                .try_reserve_exact(count.checked_mul(3).ok_or("Too many triangles")?)
                .map_err(|_| "Not enough memory")?;
            for face in data[84..].chunks_exact(50) {
                for v in face[12..48].chunks_exact(12) {
                    vertices.push(Vertex {
                        position: std::array::from_fn(|i| {
                            f32::from_le_bytes(v[i * 4..i * 4 + 4].try_into().unwrap())
                        }),
                    });
                }
            }
        } else {
            let text = std::str::from_utf8(data).map_err(|_| "Invalid or truncated binary STL")?;
            let mut words = text.split_ascii_whitespace();
            if words.next() != Some("solid") {
                return Err("Expected ASCII or binary STL".into());
            }
            // Validate the facet grammar instead of accepting arbitrary vertex lists.
            let mut lines = text.lines().map(str::trim).filter(|s| !s.is_empty());
            lines.next();
            let mut ended = false;
            while let Some(line) = lines.next() {
                if line == "endsolid" || line.starts_with("endsolid ") {
                    ended = true;
                    break;
                }
                if !line.starts_with("facet normal ") {
                    return Err("Expected facet normal".into());
                }
                if lines.next() != Some("outer loop") {
                    return Err("Expected outer loop".into());
                }
                for _ in 0..3 {
                    let mut fields = lines
                        .next()
                        .ok_or("Missing vertex")?
                        .split_ascii_whitespace();
                    if fields.next() != Some("vertex") {
                        return Err("Expected vertex".into());
                    }
                    let mut position = [0.; 3];
                    for p in &mut position {
                        *p = fields
                            .next()
                            .ok_or("Missing coordinate")?
                            .parse::<f32>()
                            .map_err(|_| "Invalid coordinate")?;
                    }
                    if fields.next().is_some() {
                        return Err("Extra vertex coordinate".into());
                    }
                    vertices.push(Vertex { position });
                }
                if lines.next() != Some("endloop") || lines.next() != Some("endfacet") {
                    return Err("Unclosed facet".into());
                }
            }
            if !ended || lines.next().is_some() {
                return Err("Missing endsolid or trailing data".into());
            }
        }
        if vertices.is_empty() {
            return Err("STL contains no triangles".into());
        }
        let mut lo = [f32::INFINITY; 3];
        let mut hi = [f32::NEG_INFINITY; 3];
        for v in &vertices {
            for i in 0..3 {
                if !v.position[i].is_finite() {
                    return Err("Non-finite coordinate".into());
                }
                lo[i] = lo[i].min(v.position[i]);
                hi[i] = hi[i].max(v.position[i]);
            }
        }
        let center = std::array::from_fn(|i| (lo[i] as f64 * 0.5 + hi[i] as f64 * 0.5) as f32);
        let radius = (0..3)
            .map(|i| ((hi[i] as f64 - lo[i] as f64) * 0.5).powi(2))
            .sum::<f64>()
            .sqrt();
        if radius == 0. || radius > f32::MAX as f64 {
            return Err("Degenerate or oversized model bounds".into());
        }
        // Normalize once on the CPU; every subsequent interaction only updates a matrix.
        for v in &mut vertices {
            for (i, value) in v.position.iter_mut().enumerate() {
                *value = ((*value as f64 - center[i] as f64) / radius) as f32;
            }
        }
        Ok(Self {
            vertices,
            center,
            radius: radius as f32,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn binary() -> Vec<u8> {
        let mut b = vec![0; 134];
        b[80..84].copy_from_slice(&1u32.to_le_bytes());
        b[108..112].copy_from_slice(&1f32.to_le_bytes());
        b[124..128].copy_from_slice(&1f32.to_le_bytes());
        b
    }
    #[test]
    fn binary_with_solid_header() {
        let mut b = binary();
        b[..5].copy_from_slice(b"solid");
        assert_eq!(Mesh::parse(&b).unwrap().vertices.len(), 3);
    }
    #[test]
    fn rejects_truncation() {
        assert!(Mesh::parse(&binary()[..133]).is_err());
    }
    #[test]
    fn rejects_nan() {
        let mut b = binary();
        b[96..100].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(Mesh::parse(&b).is_err());
    }
    #[test]
    fn ascii_and_incomplete() {
        let a = b"solid test\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\nendloop\nendfacet\nendsolid test\n";
        assert_eq!(Mesh::parse(a).unwrap().vertices.len(), 3);
        assert!(Mesh::parse(&a[..a.len() - 15]).is_err());
    }
    #[test]
    fn rejects_empty() {
        assert!(Mesh::parse(b"solid empty\nendsolid empty").is_err());
    }
}
