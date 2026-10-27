use crate::features::concept_model::{BendPoint, DiagramEdge, DiagramNode, PortSide, RelationKind};
use crate::features::concepts::{Concept, ConceptValidator};
use crate::features::information_model::{
    ClassDiagramNode, ClassRelation, InformationClass, InformationEnumeration, Multiplicity,
};
use crate::features::model::storage::StorageError;
use crate::features::model::{ModelMetadata, ModelProject};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "relation_type")]
pub enum DecomposedRelation {
    ConceptRelation {
        id: Uuid,
        from: Uuid,
        to: Uuid,
        kind: RelationKind,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source_port: Option<PortSide>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target_port: Option<PortSide>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        directed: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bends: Option<Vec<BendPoint>>,
    },
    ClassRelation {
        id: Uuid,
        from: Uuid,
        to: Uuid,
        kind: RelationKind,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source_port: Option<PortSide>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target_port: Option<PortSide>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        directed: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source_multiplicity: Option<Multiplicity>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target_multiplicity: Option<Multiplicity>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bends: Option<Vec<BendPoint>>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "diagram_type")]
pub enum DecomposedDiagram {
    ConceptDiagram {
        id: Uuid,
        name: String,
        nodes: Vec<DiagramNode>,
    },
    ClassDiagram {
        id: Uuid,
        name: String,
        nodes: Vec<ClassDiagramNode>,
    },
}

/// Sorterer JSON-nøgler deterministisk (alfabetisk) og formaterer med 2 spaces og afsluttende newline.
pub fn to_deterministic_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let mut val = serde_json::to_value(value)?;
    sort_json_value(&mut val);
    let mut json = serde_json::to_string_pretty(&val)?;
    json.push('\n');
    Ok(json)
}

fn sort_json_value(val: &mut serde_json::Value) {
    match val {
        serde_json::Value::Object(map) => {
            let sorted: BTreeMap<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| {
                    let mut v_sorted = v.clone();
                    sort_json_value(&mut v_sorted);
                    (k.clone(), v_sorted)
                })
                .collect();
            map.clear();
            for (k, v) in sorted {
                map.insert(k, v);
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                sort_json_value(item);
            }
        }
        _ => {}
    }
}

fn atomic_write_file(path: &Path, content: &str) -> Result<(), std::io::Error> {
    if path.exists() {
        if let Ok(existing) = fs::read_to_string(path) {
            if existing == content {
                return Ok(());
            }
        }
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let tmp_path = parent.join(format!(".{}.tmp", Uuid::new_v4()));
    fs::write(&tmp_path, content.as_bytes())?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}

fn cleanup_stale_files(dir: &Path, active_uuids: &HashSet<Uuid>) -> Result<(), std::io::Error> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                if let Ok(file_uuid) = Uuid::parse_str(file_stem) {
                    if !active_uuids.contains(&file_uuid) {
                        let _ = fs::remove_file(&path);
                    }
                }
            }
        }
    }
    Ok(())
}

fn deterministic_uuid(key: &str) -> Uuid {
    use std::hash::{Hash, Hasher};
    let mut h1 = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut h1);
    let u1 = h1.finish();
    let mut h2 = std::collections::hash_map::DefaultHasher::new();
    (key, "salt_kant_rel").hash(&mut h2);
    let u2 = h2.finish();
    let mut bytes = [0u8; 16];
    bytes[0..8].copy_from_slice(&u1.to_be_bytes());
    bytes[8..16].copy_from_slice(&u2.to_be_bytes());
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

/// Gemmer et ModelProject i dekomponeret format under `.kant/`.
pub fn save_decomposed(project: &ModelProject, root_path: &Path) -> Result<(), StorageError> {
    // 1. Fail-closed validering
    for concept in project.concepts() {
        ConceptValidator::validate(concept)?;
    }

    let kant_dir = if root_path.file_name().and_then(|n| n.to_str()) == Some(".kant") {
        root_path.to_path_buf()
    } else {
        root_path.join(".kant")
    };

    let concepts_dir = kant_dir.join("concepts");
    let classes_dir = kant_dir.join("classes");
    let enumerations_dir = kant_dir.join("enumerations");
    let relations_dir = kant_dir.join("relations");
    let diagrams_dir = kant_dir.join("diagrams");

    fs::create_dir_all(&concepts_dir)?;
    fs::create_dir_all(&classes_dir)?;
    fs::create_dir_all(&enumerations_dir)?;
    fs::create_dir_all(&relations_dir)?;
    fs::create_dir_all(&diagrams_dir)?;

    // 2. metadata.json
    let metadata_path = kant_dir.join("metadata.json");
    let metadata_json = to_deterministic_json(project.metadata())?;
    atomic_write_file(&metadata_path, &metadata_json)?;

    // 3. concepts/<uuid>.json
    let mut active_concept_ids = HashSet::new();
    for concept in project.concepts() {
        let cid = concept.id();
        active_concept_ids.insert(cid);
        let file_path = concepts_dir.join(format!("{}.json", cid));
        let json = to_deterministic_json(concept)?;
        atomic_write_file(&file_path, &json)?;
    }
    cleanup_stale_files(&concepts_dir, &active_concept_ids)?;

    // 4. classes/<uuid>.json
    let mut active_class_ids = HashSet::new();
    for class in project.information_model().classes() {
        let cid = class.id();
        active_class_ids.insert(cid);
        let file_path = classes_dir.join(format!("{}.json", cid));
        let json = to_deterministic_json(class)?;
        atomic_write_file(&file_path, &json)?;
    }
    cleanup_stale_files(&classes_dir, &active_class_ids)?;

    // 4b. enumerations/<uuid>.json
    let mut active_enum_ids = HashSet::new();
    for e in project.information_model().enumerations() {
        let eid = e.id();
        active_enum_ids.insert(eid);
        let file_path = enumerations_dir.join(format!("{}.json", eid));
        let json = to_deterministic_json(e)?;
        atomic_write_file(&file_path, &json)?;
    }
    cleanup_stale_files(&enumerations_dir, &active_enum_ids)?;

    // 5. relations/<uuid>.json
    let mut active_relation_ids = HashSet::new();
    for edge in project.concept_graph().edges() {
        let rel_id = edge.id().unwrap_or_else(|| {
            deterministic_uuid(&format!("concept:{}:{}", edge.from(), edge.to()))
        });
        active_relation_ids.insert(rel_id);
        let decomposed_rel = DecomposedRelation::ConceptRelation {
            id: rel_id,
            from: edge.from(),
            to: edge.to(),
            kind: edge.kind(),
            label: edge.label().map(|s| s.to_string()),
            source_port: edge.source_port(),
            target_port: edge.target_port(),
            directed: edge.directed(),
            bends: edge.bends().map(|b| b.to_vec()),
        };
        let file_path = relations_dir.join(format!("{}.json", rel_id));
        let json = to_deterministic_json(&decomposed_rel)?;
        atomic_write_file(&file_path, &json)?;
    }

    let mut info_rels: Vec<ClassRelation> = project.information_model().relations().to_vec();
    if info_rels.is_empty() && !project.information_graph().edges().is_empty() {
        for edge in project.information_graph().edges() {
            let from_class = project
                .information_graph()
                .find_node(edge.from())
                .map(|n| n.class_id())
                .unwrap_or(edge.from());
            let to_class = project
                .information_graph()
                .find_node(edge.to())
                .map(|n| n.class_id())
                .unwrap_or(edge.to());
            let mut rel = ClassRelation::with_multiplicities(
                from_class,
                to_class,
                edge.kind(),
                edge.label().map(String::from),
                edge.source_port(),
                edge.target_port(),
                edge.directed(),
                edge.source_multiplicity(),
                edge.target_multiplicity(),
            );
            if let Some(b) = edge.bends() {
                rel.set_bends(Some(b.to_vec()));
            }
            info_rels.push(rel);
        }
    }

    for rel in info_rels {
        let rel_id = rel.id();
        active_relation_ids.insert(rel_id);
        let decomposed_rel = DecomposedRelation::ClassRelation {
            id: rel_id,
            from: rel.from_class(),
            to: rel.to_class(),
            kind: rel.kind(),
            label: rel.label().map(|s| s.to_string()),
            source_port: rel.source_port(),
            target_port: rel.target_port(),
            directed: rel.directed(),
            source_multiplicity: rel.source_multiplicity(),
            target_multiplicity: rel.target_multiplicity(),
            bends: rel.bends().map(|b| b.to_vec()),
        };
        let file_path = relations_dir.join(format!("{}.json", rel_id));
        let json = to_deterministic_json(&decomposed_rel)?;
        atomic_write_file(&file_path, &json)?;
    }
    cleanup_stale_files(&relations_dir, &active_relation_ids)?;

    // 6. diagrams/<uuid>.json
    let mut active_diagram_ids = HashSet::new();
    let concept_diag_id = Uuid::from_u128(1);
    active_diagram_ids.insert(concept_diag_id);
    let mut concept_nodes = project.concept_graph().nodes().to_vec();
    concept_nodes.sort_by_key(|n| n.id());
    let concept_diagram = DecomposedDiagram::ConceptDiagram {
        id: concept_diag_id,
        name: "Begrebsmodel".to_string(),
        nodes: concept_nodes,
    };
    let c_diag_path = diagrams_dir.join(format!("{}.json", concept_diag_id));
    let c_diag_json = to_deterministic_json(&concept_diagram)?;
    atomic_write_file(&c_diag_path, &c_diag_json)?;

    let class_diag_id = Uuid::from_u128(2);
    active_diagram_ids.insert(class_diag_id);
    let mut class_nodes = project.information_graph().nodes().to_vec();
    class_nodes.sort_by_key(|n| n.id());
    let class_diagram = DecomposedDiagram::ClassDiagram {
        id: class_diag_id,
        name: "Informationsmodel".to_string(),
        nodes: class_nodes,
    };
    let cl_diag_path = diagrams_dir.join(format!("{}.json", class_diag_id));
    let cl_diag_json = to_deterministic_json(&class_diagram)?;
    atomic_write_file(&cl_diag_path, &cl_diag_json)?;

    cleanup_stale_files(&diagrams_dir, &active_diagram_ids)?;

    Ok(())
}

/// Indlæser et ModelProject fra et dekomponeret katalog under `.kant/`.
pub fn load_decomposed(root_path: &Path) -> Result<ModelProject, StorageError> {
    let kant_dir = if root_path.file_name().and_then(|n| n.to_str()) == Some(".kant") {
        root_path.to_path_buf()
    } else {
        root_path.join(".kant")
    };

    if !kant_dir.is_dir() {
        return Err(StorageError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Mappen {:?} findes ikke", kant_dir),
        )));
    }

    // 1. metadata.json
    let metadata_path = kant_dir.join("metadata.json");
    let metadata_content = fs::read_to_string(&metadata_path)?;
    let metadata: ModelMetadata = serde_json::from_str(&metadata_content)?;
    let mut project = ModelProject::new(metadata);

    // 2. concepts/*.json
    let concepts_dir = kant_dir.join("concepts");
    if concepts_dir.is_dir() {
        for entry in fs::read_dir(concepts_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                let concept: Concept = serde_json::from_str(&content)?;
                ConceptValidator::validate(&concept)?;
                project.concepts_mut().push(concept);
            }
        }
    }
    project.sort_concepts_alphabetically();

    // 3. classes/*.json
    let classes_dir = kant_dir.join("classes");
    if classes_dir.is_dir() {
        for entry in fs::read_dir(classes_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                let class: InformationClass = serde_json::from_str(&content)?;
                project.information_model_mut().add_class(class);
            }
        }
    }

    // 3b. enumerations/*.json
    let enumerations_dir = kant_dir.join("enumerations");
    if enumerations_dir.is_dir() {
        for entry in fs::read_dir(enumerations_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                let enumeration: InformationEnumeration = serde_json::from_str(&content)?;
                project.information_model_mut().add_enumeration(enumeration);
            }
        }
    }

    // 4. diagrams/*.json
    let diagrams_dir = kant_dir.join("diagrams");
    if diagrams_dir.is_dir() {
        for entry in fs::read_dir(diagrams_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                if let Ok(diagram) = serde_json::from_str::<DecomposedDiagram>(&content) {
                    match diagram {
                        DecomposedDiagram::ConceptDiagram { nodes, .. } => {
                            *project.concept_graph_mut().nodes_mut() = nodes;
                        }
                        DecomposedDiagram::ClassDiagram { nodes, .. } => {
                            *project.information_graph_mut().nodes_mut() = nodes;
                        }
                    }
                }
            }
        }
    }

    // 5. relations/*.json
    let relations_dir = kant_dir.join("relations");
    if relations_dir.is_dir() {
        for entry in fs::read_dir(relations_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                if let Ok(rel) = serde_json::from_str::<DecomposedRelation>(&content) {
                    match rel {
                        DecomposedRelation::ConceptRelation {
                            id,
                            from,
                            to,
                            kind,
                            label,
                            source_port,
                            target_port,
                            directed,
                            bends,
                        } => {
                            let mut edge = DiagramEdge::with_label(from, to, kind, label);
                            edge.set_id(id);
                            edge.set_ports(source_port, target_port);
                            if let Some(d) = directed {
                                edge.set_directed(d);
                            }
                            if let Some(b) = bends {
                                edge.set_bends(Some(b));
                            }
                            project.concept_graph_mut().edges_mut().push(edge);
                        }
                        DecomposedRelation::ClassRelation {
                            id,
                            from,
                            to,
                            kind,
                            label,
                            source_port,
                            target_port,
                            directed,
                            source_multiplicity,
                            target_multiplicity,
                            bends,
                        } => {
                            let mut rel = ClassRelation::with_multiplicities(
                                from,
                                to,
                                kind,
                                label,
                                source_port,
                                target_port,
                                directed,
                                source_multiplicity,
                                target_multiplicity,
                            );
                            rel.set_id(id);
                            if let Some(b) = bends {
                                rel.set_bends(Some(b));
                            }
                            project.information_model_mut().add_relation(rel);
                        }
                    }
                }
            }
        }
    }

    project.sync_information_graph();

    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_to_deterministic_json_sorts_keys_and_appends_newline() {
        let val = json!({
            "zebra": 1,
            "apple": 2,
            "nested": {
                "banana": true,
                "aardvark": "test"
            }
        });

        let json_str = to_deterministic_json(&val).unwrap();
        assert!(json_str.ends_with('\n'), "Skal have afsluttende newline");

        // "apple" skal komme før "nested", og "nested" før "zebra"
        let pos_apple = json_str.find("\"apple\"").unwrap();
        let pos_nested = json_str.find("\"nested\"").unwrap();
        let pos_zebra = json_str.find("\"zebra\"").unwrap();
        assert!(pos_apple < pos_nested);
        assert!(pos_nested < pos_zebra);

        // "aardvark" skal komme før "banana"
        let pos_aardvark = json_str.find("\"aardvark\"").unwrap();
        let pos_banana = json_str.find("\"banana\"").unwrap();
        assert!(pos_aardvark < pos_banana);
    }

    #[test]
    fn test_deterministic_uuid_stability() {
        let u1 = deterministic_uuid("test_key_1");
        let u2 = deterministic_uuid("test_key_1");
        let u3 = deterministic_uuid("test_key_2");

        assert_eq!(u1, u2, "Samme nøgle skal producere identisk UUID");
        assert_ne!(
            u1, u3,
            "Forskellige nøgler skal producere forskellige UUIDs"
        );
    }
}
