use crate::features::concept_model::{PortSide, RelationKind};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveType {
    CharacterString,
    Integer,
    Decimal,
    Boolean,
    Date,
    DateTime,
    Time,
    Uri,
}

#[rustfmt::skip]
impl PrimitiveType {
    pub const ALL: &'static [PrimitiveType] = &[
        Self::CharacterString, Self::Integer, Self::Decimal, Self::Boolean,
        Self::Date, Self::DateTime, Self::Time, Self::Uri,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CharacterString => "CharacterString",
            Self::Integer => "Integer",
            Self::Decimal => "Decimal",
            Self::Boolean => "Boolean",
            Self::Date => "Date",
            Self::DateTime => "DateTime",
            Self::Time => "Time",
            Self::Uri => "URI",
        }
    }
}

impl std::fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Multiplicity {
    lower: u32,
    upper: Option<u32>,
}

#[rustfmt::skip]
impl Multiplicity {
    pub const PRESETS: &'static [Multiplicity] = &[
        Multiplicity { lower: 1, upper: Some(1) },
        Multiplicity { lower: 0, upper: Some(1) },
        Multiplicity { lower: 0, upper: None },
        Multiplicity { lower: 1, upper: None },
    ];

    pub fn new(lower: u32, upper: Option<u32>) -> Self { Self { lower, upper } }
    pub fn exactly_one() -> Self { Self::new(1, Some(1)) }
    pub fn zero_or_one() -> Self { Self::new(0, Some(1)) }
    pub fn zero_or_more() -> Self { Self::new(0, None) }
    pub fn one_or_more() -> Self { Self::new(1, None) }
    pub fn lower(&self) -> u32 { self.lower }
    pub fn upper(&self) -> Option<u32> { self.upper }

    pub fn to_display_string(&self) -> String {
        match self.upper {
            Some(u) => if self.lower == u { format!("{}", self.lower) } else { format!("{}..{}", self.lower, u) },
            None => format!("{}..*", self.lower),
        }
    }
}

impl std::fmt::Display for Multiplicity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_display_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InformationDataType {
    Primitive(PrimitiveType),
    Enumeration { enumeration_id: Uuid },
    Structured { structured_id: Uuid },
}

#[rustfmt::skip]
impl From<PrimitiveType> for InformationDataType {
    fn from(p: PrimitiveType) -> Self { Self::Primitive(p) }
}
impl Default for InformationDataType {
    fn default() -> Self {
        Self::Primitive(PrimitiveType::CharacterString)
    }
}
#[rustfmt::skip]
impl PartialEq<PrimitiveType> for InformationDataType {
    fn eq(&self, other: &PrimitiveType) -> bool { match self { Self::Primitive(p) => p == other, _ => false } }
}
#[rustfmt::skip]
impl PartialEq<InformationDataType> for PrimitiveType {
    fn eq(&self, other: &InformationDataType) -> bool { other == self }
}
#[rustfmt::skip]
impl PartialEq<PrimitiveType> for &InformationDataType {
    fn eq(&self, other: &PrimitiveType) -> bool { *self == other }
}
#[rustfmt::skip]
impl PartialEq<&InformationDataType> for PrimitiveType {
    fn eq(&self, other: &&InformationDataType) -> bool { *other == self }
}

impl InformationDataType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Primitive(p) => p.as_str(),
            Self::Enumeration { .. } => "«enumeration»",
            Self::Structured { .. } => "«dataType»",
        }
    }

    pub fn display_name(&self, model: &InformationModel) -> String {
        match self {
            Self::Primitive(p) => p.as_str().to_string(),
            Self::Enumeration { enumeration_id } => model
                .get_enumeration(*enumeration_id)
                .map(|e| e.name().to_string())
                .unwrap_or_else(|| "«enumeration»".to_string()),
            Self::Structured { structured_id } => model
                .get_structured_type(*structured_id)
                .map(|dt| dt.name().to_string())
                .unwrap_or_else(|| "«dataType»".to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InformationEnumeration {
    id: Uuid,
    name: String,
    #[serde(default)]
    definition: Option<String>,
    #[serde(default)]
    values: Vec<String>,
}

#[rustfmt::skip]
impl InformationEnumeration {
    pub fn new(name: impl Into<String>, values: Vec<String>) -> Self {
        Self::new_with_id(Uuid::new_v4(), name, values)
    }

    pub fn new_with_id(id: Uuid, name: impl Into<String>, values: Vec<String>) -> Self {
        Self { id, name: name.into(), definition: None, values }
    }

    pub fn id(&self) -> Uuid { self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn set_name(&mut self, name: impl Into<String>) { self.name = name.into(); }
    pub fn definition(&self) -> Option<&str> { self.definition.as_deref() }
    pub fn set_definition(&mut self, definition: Option<String>) { self.definition = definition; }
    pub fn values(&self) -> &[String] { &self.values }
    pub fn values_mut(&mut self) -> &mut Vec<String> { &mut self.values }

    pub fn add_value(&mut self, val: impl Into<String>) {
        let v = val.into();
        if !self.values.contains(&v) { self.values.push(v); }
    }

    pub fn remove_value(&mut self, val: &str) {
        self.values.retain(|v| v != val);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuredDataType {
    id: Uuid,
    name: String,
    #[serde(default)]
    definition: Option<String>,
    #[serde(default)]
    attributes: Vec<Attribute>,
}

#[rustfmt::skip]
impl StructuredDataType {
    pub fn new(name: impl Into<String>) -> Self {
        Self::new_with_id(Uuid::new_v4(), name)
    }

    pub fn new_with_id(id: Uuid, name: impl Into<String>) -> Self {
        Self { id, name: name.into(), definition: None, attributes: Vec::new() }
    }

    pub fn id(&self) -> Uuid { self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn set_name(&mut self, name: impl Into<String>) { self.name = name.into(); }
    pub fn definition(&self) -> Option<&str> { self.definition.as_deref() }
    pub fn set_definition(&mut self, definition: Option<String>) { self.definition = definition; }
    pub fn attributes(&self) -> &[Attribute] { &self.attributes }
    pub fn attributes_mut(&mut self) -> &mut Vec<Attribute> { &mut self.attributes }
    pub fn add_attribute(&mut self, attribute: Attribute) { self.attributes.push(attribute); }

    pub fn remove_attribute(&mut self, attr_id: Uuid) -> Option<Attribute> {
        self.attributes.iter().position(|a| a.id() == attr_id).map(|pos| self.attributes.remove(pos))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attribute {
    id: Uuid,
    name: String,
    data_type: InformationDataType,
    multiplicity: Multiplicity,
    #[serde(default)]
    concept_ids: Vec<Uuid>,
}

#[rustfmt::skip]
impl Attribute {
    pub fn new(
        name: impl Into<String>,
        data_type: impl Into<InformationDataType>,
        multiplicity: Multiplicity,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            data_type: data_type.into(),
            multiplicity,
            concept_ids: Vec::new(),
        }
    }

    pub fn with_concepts(mut self, ids: Vec<Uuid>) -> Self { self.concept_ids = ids; self }
    pub fn id(&self) -> Uuid { self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn set_name(&mut self, name: impl Into<String>) { self.name = name.into(); }
    pub fn data_type(&self) -> &InformationDataType { &self.data_type }
    pub fn set_data_type(&mut self, data_type: impl Into<InformationDataType>) { self.data_type = data_type.into(); }
    pub fn primitive_type(&self) -> Option<PrimitiveType> {
        match self.data_type { InformationDataType::Primitive(p) => Some(p), _ => None }
    }
    pub fn multiplicity(&self) -> Multiplicity { self.multiplicity }
    pub fn set_multiplicity(&mut self, multiplicity: Multiplicity) { self.multiplicity = multiplicity; }
    pub fn concept_ids(&self) -> &[Uuid] { &self.concept_ids }
    pub fn add_concept_id(&mut self, id: Uuid) {
        if !self.concept_ids.contains(&id) { self.concept_ids.push(id); }
    }
    pub fn remove_concept_id(&mut self, id: Uuid) { self.concept_ids.retain(|c| *c != id); }
    pub fn set_concept_ids(&mut self, ids: Vec<Uuid>) { self.concept_ids = ids; }
    pub fn clear_concept_ids(&mut self) { self.concept_ids.clear(); }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InformationClass {
    id: Uuid,
    name: String,
    description: Option<String>,
    #[serde(default)]
    concept_ids: Vec<Uuid>,
    #[serde(default)]
    attributes: Vec<Attribute>,
    #[serde(default)]
    is_abstract: bool,
    #[serde(default = "default_true")]
    is_local: bool,
    #[serde(default)]
    origin_model: Option<String>,
}

#[rustfmt::skip]
impl InformationClass {
    pub fn new(name: impl Into<String>) -> Self {
        Self::new_with_id(Uuid::new_v4(), name)
    }

    pub fn new_with_id(id: Uuid, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            description: None,
            concept_ids: Vec::new(),
            attributes: Vec::new(),
            is_abstract: false,
            is_local: true,
            origin_model: None,
        }
    }

    pub fn from_concept(concept: &crate::features::concepts::Concept) -> Self {
        let (is_local, origin_model) = match concept.belongs_to_domain() {
            crate::features::concepts::BelongsToDomain::Yes => (true, None),
            crate::features::concepts::BelongsToDomain::No => (false, None),
            crate::features::concepts::BelongsToDomain::ModelRef(uri) => (false, Some(uri.clone())),
        };

        Self {
            id: Uuid::new_v4(),
            name: concept.preferred_term().to_string(),
            description: if concept.definition().is_empty() { None } else { Some(concept.definition().to_string()) },
            concept_ids: vec![concept.id()],
            attributes: Vec::new(),
            is_abstract: false,
            is_local,
            origin_model,
        }
    }

    pub fn id(&self) -> Uuid { self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn set_name(&mut self, name: impl Into<String>) { self.name = name.into(); }
    pub fn is_abstract(&self) -> bool { self.is_abstract }
    pub fn set_abstract(&mut self, is_abstract: bool) { self.is_abstract = is_abstract; }
    pub fn is_local(&self) -> bool { self.is_local }
    pub fn set_local(&mut self, is_local: bool) { self.is_local = is_local; }
    pub fn origin_model(&self) -> Option<&str> { self.origin_model.as_deref() }
    pub fn set_origin_model(&mut self, origin: Option<String>) { self.origin_model = origin; }
    pub fn description(&self) -> Option<&str> { self.description.as_deref() }
    pub fn set_description(&mut self, desc: Option<String>) { self.description = desc; }
    pub fn concept_ids(&self) -> &[Uuid] { &self.concept_ids }
    pub fn add_concept_id(&mut self, id: Uuid) {
        if !self.concept_ids.contains(&id) { self.concept_ids.push(id); }
    }
    pub fn remove_concept_id(&mut self, id: Uuid) { self.concept_ids.retain(|c| *c != id); }
    pub fn attributes(&self) -> &[Attribute] { &self.attributes }
    pub fn attributes_mut(&mut self) -> &mut Vec<Attribute> { &mut self.attributes }
    pub fn add_attribute(&mut self, attribute: Attribute) { self.attributes.push(attribute); }

    pub fn remove_attribute(&mut self, attr_id: Uuid) -> Option<Attribute> {
        self.attributes.iter().position(|a| a.id() == attr_id).map(|pos| self.attributes.remove(pos))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassRelation {
    #[serde(default = "Uuid::new_v4")]
    id: Uuid,
    from_class: Uuid,
    to_class: Uuid,
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
}

#[rustfmt::skip]
impl ClassRelation {
    pub fn new(from_class: Uuid, to_class: Uuid, kind: RelationKind, label: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_class,
            to_class,
            kind,
            label,
            source_port: None,
            target_port: None,
            directed: if kind == RelationKind::Association { Some(true) } else { None },
            source_multiplicity: None,
            target_multiplicity: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_multiplicities(
        from_class: Uuid,
        to_class: Uuid,
        kind: RelationKind,
        label: Option<String>,
        source_port: Option<PortSide>,
        target_port: Option<PortSide>,
        directed: Option<bool>,
        source_multiplicity: Option<Multiplicity>,
        target_multiplicity: Option<Multiplicity>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_class,
            to_class,
            kind,
            label,
            source_port,
            target_port,
            directed,
            source_multiplicity,
            target_multiplicity,
        }
    }

    pub fn id(&self) -> Uuid { self.id }
    pub fn set_id(&mut self, id: Uuid) { self.id = id; }
    pub fn from_class(&self) -> Uuid { self.from_class }
    pub fn to_class(&self) -> Uuid { self.to_class }
    pub fn kind(&self) -> RelationKind { self.kind }
    pub fn set_kind(&mut self, kind: RelationKind) {
        self.kind = kind;
        if kind == RelationKind::Association && self.directed.is_none() { self.directed = Some(true); }
    }
    pub fn label(&self) -> Option<&str> { self.label.as_deref() }
    pub fn set_label(&mut self, label: Option<String>) { self.label = label; }
    pub fn source_port(&self) -> Option<PortSide> { self.source_port }
    pub fn target_port(&self) -> Option<PortSide> { self.target_port }
    pub fn set_ports(&mut self, source_port: Option<PortSide>, target_port: Option<PortSide>) {
        self.source_port = source_port;
        self.target_port = target_port;
    }
    pub fn is_directed(&self) -> bool {
        if self.kind == RelationKind::Association { self.directed.unwrap_or(true) } else { false }
    }
    pub fn directed(&self) -> Option<bool> { self.directed }
    pub fn set_directed(&mut self, directed: bool) { self.directed = Some(directed); }
    pub fn source_multiplicity(&self) -> Option<Multiplicity> { self.source_multiplicity }
    pub fn set_source_multiplicity(&mut self, mult: Option<Multiplicity>) { self.source_multiplicity = mult; }
    pub fn target_multiplicity(&self) -> Option<Multiplicity> { self.target_multiplicity }
    pub fn set_target_multiplicity(&mut self, mult: Option<Multiplicity>) { self.target_multiplicity = mult; }

    pub fn reverse(&mut self) {
        std::mem::swap(&mut self.from_class, &mut self.to_class);
        std::mem::swap(&mut self.source_port, &mut self.target_port);
        std::mem::swap(&mut self.source_multiplicity, &mut self.target_multiplicity);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct InformationModel {
    classes: Vec<InformationClass>,
    #[serde(default)]
    enumerations: Vec<InformationEnumeration>,
    #[serde(default)]
    structured_types: Vec<StructuredDataType>,
    #[serde(default)]
    relations: Vec<ClassRelation>,
}

#[rustfmt::skip]
impl InformationModel {
    pub fn new() -> Self {
        Self {
            classes: Vec::new(),
            enumerations: Vec::new(),
            structured_types: Vec::new(),
            relations: Vec::new(),
        }
    }

    pub fn relations(&self) -> &[ClassRelation] { &self.relations }
    pub fn relations_mut(&mut self) -> &mut Vec<ClassRelation> { &mut self.relations }

    pub fn add_relation(&mut self, relation: ClassRelation) -> Uuid {
        let id = relation.id();
        if let Some(existing) = self.relations.iter_mut().find(|r| {
            r.from_class == relation.from_class && r.to_class == relation.to_class && r.kind == relation.kind
        }) {
            *existing = relation;
            return existing.id();
        }
        self.relations.push(relation);
        id
    }

    pub fn remove_relation(&mut self, from_class: Uuid, to_class: Uuid) -> bool {
        let prev_len = self.relations.len();
        self.relations.retain(|r| !(r.from_class == from_class && r.to_class == to_class));
        self.relations.len() < prev_len
    }

    pub fn find_relation(&self, from_class: Uuid, to_class: Uuid) -> Option<&ClassRelation> {
        self.relations.iter().find(|r| r.from_class == from_class && r.to_class == to_class)
    }

    pub fn find_relation_mut(&mut self, from_class: Uuid, to_class: Uuid) -> Option<&mut ClassRelation> {
        self.relations.iter_mut().find(|r| r.from_class == from_class && r.to_class == to_class)
    }

    pub fn relations_for_class(&self, class_id: Uuid) -> Vec<&ClassRelation> {
        self.relations.iter().filter(|r| r.from_class == class_id || r.to_class == class_id).collect()
    }

    pub fn remove_relations_for_class(&mut self, class_id: Uuid) {
        self.relations.retain(|r| r.from_class != class_id && r.to_class != class_id);
    }

    pub fn update_relation_kind(&mut self, from_class: Uuid, to_class: Uuid, kind: RelationKind) -> bool {
        self.find_relation_mut(from_class, to_class).map(|r| r.set_kind(kind)).is_some()
    }

    pub fn update_relation_label(&mut self, from_class: Uuid, to_class: Uuid, label: Option<String>) -> bool {
        self.find_relation_mut(from_class, to_class).map(|r| r.set_label(label)).is_some()
    }

    pub fn update_relation_source_multiplicity(&mut self, from: Uuid, to: Uuid, mult: Option<Multiplicity>) -> bool {
        self.find_relation_mut(from, to).map(|r| r.set_source_multiplicity(mult)).is_some()
    }

    pub fn update_relation_target_multiplicity(&mut self, from: Uuid, to: Uuid, mult: Option<Multiplicity>) -> bool {
        self.find_relation_mut(from, to).map(|r| r.set_target_multiplicity(mult)).is_some()
    }

    pub fn update_relation_directed(&mut self, from_class: Uuid, to_class: Uuid, directed: bool) -> bool {
        self.find_relation_mut(from_class, to_class).map(|r| r.set_directed(directed)).is_some()
    }

    pub fn update_relation_ports(&mut self, from: Uuid, to: Uuid, src: Option<PortSide>, tgt: Option<PortSide>) -> bool {
        self.find_relation_mut(from, to).map(|r| r.set_ports(src, tgt)).is_some()
    }

    pub fn reverse_relation(&mut self, from_class: Uuid, to_class: Uuid) -> bool {
        if let Some(pos) = self.relations.iter().position(|r| r.from_class == from_class && r.to_class == to_class) {
            self.relations[pos].reverse();
            true
        } else {
            false
        }
    }

    pub fn classes(&self) -> &[InformationClass] { &self.classes }
    pub fn classes_mut(&mut self) -> &mut Vec<InformationClass> { &mut self.classes }
    pub fn get_class(&self, id: Uuid) -> Option<&InformationClass> { self.classes.iter().find(|c| c.id() == id) }
    pub fn get_class_mut(&mut self, id: Uuid) -> Option<&mut InformationClass> { self.classes.iter_mut().find(|c| c.id() == id) }

    pub fn add_class(&mut self, class: InformationClass) -> Uuid {
        let id = class.id();
        self.classes.push(class);
        id
    }

    pub fn remove_class(&mut self, id: Uuid) -> Option<InformationClass> {
        self.remove_relations_for_class(id);
        self.classes.iter().position(|c| c.id() == id).map(|pos| self.classes.remove(pos))
    }

    pub fn enumerations(&self) -> &[InformationEnumeration] { &self.enumerations }
    pub fn enumerations_mut(&mut self) -> &mut Vec<InformationEnumeration> { &mut self.enumerations }
    pub fn get_enumeration(&self, id: Uuid) -> Option<&InformationEnumeration> { self.enumerations.iter().find(|e| e.id() == id) }
    pub fn get_enumeration_mut(&mut self, id: Uuid) -> Option<&mut InformationEnumeration> { self.enumerations.iter_mut().find(|e| e.id() == id) }

    pub fn add_enumeration(&mut self, enumeration: InformationEnumeration) -> Uuid {
        let id = enumeration.id();
        self.enumerations.push(enumeration);
        id
    }

    pub fn remove_enumeration(&mut self, id: Uuid) -> Option<InformationEnumeration> {
        self.remove_relations_for_class(id);
        self.enumerations.iter().position(|e| e.id() == id).map(|pos| self.enumerations.remove(pos))
    }

    pub fn structured_types(&self) -> &[StructuredDataType] { &self.structured_types }
    pub fn structured_types_mut(&mut self) -> &mut Vec<StructuredDataType> { &mut self.structured_types }
    pub fn get_structured_type(&self, id: Uuid) -> Option<&StructuredDataType> { self.structured_types.iter().find(|dt| dt.id() == id) }
    pub fn get_structured_type_mut(&mut self, id: Uuid) -> Option<&mut StructuredDataType> { self.structured_types.iter_mut().find(|dt| dt.id() == id) }

    pub fn add_structured_type(&mut self, structured_type: StructuredDataType) -> Uuid {
        let id = structured_type.id();
        self.structured_types.push(structured_type);
        id
    }

    pub fn remove_structured_type(&mut self, id: Uuid) -> Option<StructuredDataType> {
        self.structured_types.iter().position(|dt| dt.id() == id).map(|pos| self.structured_types.remove(pos))
    }
}
