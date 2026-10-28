//! Format adapters feed the same triangle stream to the existing renderer.
use crate::mesh::{Mesh, Vertex};
use std::{io::Cursor, path::Path};

pub fn parse(path: &Path, data: &[u8]) -> Result<Mesh, String> {
    match path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "obj" => obj(data),
        "3mf" => three_mf(data).map_err(|e| format!("3MF: {e}")),
        "stl" | "" => Mesh::parse(data),
        ext => Err(format!(
            "Unsupported model format: {ext}; expected STL, 3MF or OBJ"
        )),
    }
}

fn obj(data: &[u8]) -> Result<Mesh, String> {
    let options = tobj::LoadOptions {
        triangulate: true,
        ignore_points: true,
        ignore_lines: true,
        ..Default::default()
    };
    // Geometry-only viewer: do not require or read external MTL/texture files.
    let (models, _) = tobj::load_obj_buf(&mut Cursor::new(data), &options, |_| {
        Ok((Vec::new(), Default::default()))
    })
    .map_err(|e| format!("OBJ: {e}"))?;
    let mut vertices = Vec::new();
    for model in models {
        vertices
            .try_reserve(model.mesh.indices.len())
            .map_err(|_| "Not enough memory")?;
        for index in model.mesh.indices {
            let offset = (index as usize).checked_mul(3).ok_or("Invalid OBJ index")?;
            let p = model
                .mesh
                .positions
                .get(offset..offset + 3)
                .ok_or("Invalid OBJ index")?;
            vertices.push(Vertex {
                position: [p[0], p[1], p[2]],
            });
        }
    }
    Mesh::from_vertices(vertices, "unspecified (OBJ)".into())
}

fn geometry_model_path(xml: &[u8]) -> Result<String, String> {
    let relationships =
        lib3mf_core::archive::opc::parse_relationships(xml).map_err(|e| e.to_string())?;
    let mut models = relationships
        .into_iter()
        .filter(|r| r.rel_type == "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel");
    let model = models.next().ok_or("Missing 3MF model relationship")?;
    if models.next().is_some() {
        return Err("Multiple 3MF model relationships".into());
    }
    if !model.target_mode.eq_ignore_ascii_case("internal") {
        return Err("External 3MF model relationships are not supported".into());
    }
    Ok(model.target.trim_start_matches('/').to_string())
}

fn three_mf(data: &[u8]) -> Result<Mesh, String> {
    use lib3mf_core::{
        archive::{ArchiveReader, ZipArchiver},
        model::{ResolveOptions, Unit, resolver::PartResolver},
        parser::parse_model,
    };
    let mut archive = ZipArchiver::new(Cursor::new(data)).map_err(|e| e.to_string())?;
    // Only geometry relationships are required by this viewer. Some slicers
    // leave dangling thumbnail references; those must not prevent viewing.
    let relationships = archive
        .read_entry("_rels/.rels")
        .map_err(|e| e.to_string())?;
    let path = geometry_model_path(&relationships)?;
    let xml = archive.read_entry(&path).map_err(|e| e.to_string())?;
    let model = parse_model(Cursor::new(xml)).map_err(|e| e.to_string())?;
    let unit = model.unit;
    let mut resolver = PartResolver::new(&mut archive, model);
    let instances = resolver
        .resolve_meshes(&ResolveOptions::default())
        .map_err(|e| e.to_string())?;
    let mut vertices = Vec::new();
    for instance in instances {
        // The resolver accumulates transforms in source coordinates. Mixed-unit
        // component trees need unit-aware composition rather than a final scale.
        if instance.unit != unit {
            return Err("Mixed units across model parts are not supported".into());
        }
        let count = instance
            .mesh
            .triangles
            .len()
            .checked_mul(3)
            .ok_or("Too many triangles")?;
        vertices
            .try_reserve(count)
            .map_err(|_| "Not enough memory")?;
        let scale = unit.convert(1.0, Unit::Millimeter);
        for triangle in &instance.mesh.triangles {
            for index in [triangle.v1, triangle.v2, triangle.v3] {
                let vertex = instance
                    .mesh
                    .vertices
                    .get(index as usize)
                    .ok_or("Invalid triangle vertex index")?;
                let p = instance
                    .transform
                    .transform_point3([vertex.x, vertex.y, vertex.z].into());
                vertices.push(Vertex {
                    position: p.to_array().map(|x| (f64::from(x) * scale) as f32),
                });
            }
        }
    }
    Mesh::from_vertices(vertices, "millimeter (3MF)".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn obj_negative_indices_quad_and_missing_material() {
        let mesh = parse(
            Path::new("QUAD.OBJ"),
            include_bytes!("../tests/fixtures/quad.obj"),
        )
        .unwrap();
        assert_eq!(mesh.vertices.len(), 6);
        assert_eq!(mesh.bounds, [[0., 0., 0.], [1., 1., 0.]]);
        assert!(mesh.units.contains("unspecified"));
    }

    #[test]
    fn build_component_transforms_instances_and_units() {
        let mesh = three_mf(include_bytes!("../tests/fixtures/instances.3mf")).unwrap();
        assert_eq!(mesh.vertices.len(), 6);
        for (actual, expected) in mesh.bounds[1].into_iter().zip([101.6, 76.2, 0.]) {
            assert!((actual - expected).abs() < 1e-4, "{actual} != {expected}");
        }
        assert_eq!(mesh.bounds[0], [0., 0., 0.]);
        assert_eq!(mesh.units, "millimeter (3MF)");
    }

    #[test]
    fn external_component() {
        let mesh = three_mf(include_bytes!("../tests/fixtures/external.3mf")).unwrap();
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.bounds[1], [1., 1., 0.]);
    }

    #[test]
    fn missing_thumbnail_does_not_block_geometry() {
        let mesh = three_mf(include_bytes!("../tests/fixtures/missing-thumbnail.3mf")).unwrap();
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.bounds[1], [1., 1., 0.]);
    }

    #[test]
    fn rejects_external_and_ambiguous_model_relationships() {
        let relationship = r#"<Relationship Target="model.model" TargetMode="External" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>"#;
        assert!(geometry_model_path(relationship.as_bytes()).is_err());
        let internal = relationship.replace("External", "Internal");
        assert!(
            geometry_model_path(
                format!("<Relationships>{internal}{internal}</Relationships>").as_bytes()
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_invalid_models() {
        assert!(three_mf(include_bytes!("../tests/fixtures/cycle.3mf")).is_err());
        assert!(three_mf(include_bytes!("../tests/fixtures/bad-index.3mf")).is_err());
        assert!(three_mf(b"invalid zip").is_err());
        for data in [
            b"v 0 0 0\nf 1 2 3\n".as_slice(),
            b"v NaN 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n",
            b"",
        ] {
            assert!(obj(data).is_err());
        }
    }
}
