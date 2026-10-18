use crate::features::concepts::Concept;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod ui;

pub type NodeId = Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationKind {
    Generalization,
    Association,
    Composition,
    Dependency,
}

impl RelationKind {
    pub const ALL: &'static [RelationKind] = &[
        Self::Generalization,
        Self::Association,
        Self::Composition,
        Self::Dependency,
    ];
    pub const CONCEPT_RELATIONS: &'static [RelationKind] =
        &[Self::Generalization, Self::Association];

    pub fn is_dependency(self) -> bool {
        matches!(self, Self::Dependency)
    }
}

impl std::fmt::Display for RelationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Generalization => write!(f, "Generalisering (UML trekant)"),
            Self::Association => write!(f, "Association (UML linje)"),
            Self::Composition => write!(f, "Komposition"),
            Self::Dependency => write!(f, "Dependency (stiplet pil)"),
        }
    }
}

/// De fire forbindelsesporte på en rektangulær diagram-node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortSide {
    Top,
    Right,
    Bottom,
    Left,
}

impl PortSide {
    /// Udadgående enheds-normalvektor for portsiden.
    pub fn normal(self) -> (f32, f32) {
        match self {
            Self::Top => (0.0, -1.0),
            Self::Bottom => (0.0, 1.0),
            Self::Left => (-1.0, 0.0),
            Self::Right => (1.0, 0.0),
        }
    }

    /// Hvorvidt denne port forbinder vertikalt (Top eller Bund).
    pub fn is_vertical(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }

    /// Modstående portside.
    pub fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

pub const GRID_SIZE: f32 = 20.0;
pub const DEFAULT_NODE_WIDTH: f32 = 180.0;
pub const DEFAULT_NODE_HEIGHT: f32 = 80.0;

fn default_node_width() -> f32 {
    DEFAULT_NODE_WIDTH
}

fn default_node_height() -> f32 {
    DEFAULT_NODE_HEIGHT
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagramNode {
    id: NodeId,
    concept_id: Uuid,
    label: String,
    #[serde(default = "default_is_local")]
    is_local: bool,
    x: f32,
    y: f32,
    #[serde(default = "default_node_width")]
    width: f32,
    #[serde(default = "default_node_height")]
    height: f32,
}

fn default_is_local() -> bool {
    true
}

impl DiagramNode {
    pub fn new(concept: &Concept, x: f32, y: f32) -> Self {
        Self {
            id: concept.id(),
            concept_id: concept.id(),

            label: concept.preferred_term().to_string(),
            is_local: concept.belongs_to_domain().is_local(),
            x,
            y,
            width: DEFAULT_NODE_WIDTH,
            height: DEFAULT_NODE_HEIGHT,
        }
    }

    pub fn custom(id: NodeId, label: String, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            id,
            concept_id: Uuid::nil(),
            label,
            is_local: true,
            x,
            y,
            width,
            height,
        }
    }

    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn concept_id(&self) -> Uuid {
        self.concept_id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn set_label(&mut self, label: String) {
        self.label = label;
    }

    pub fn is_local(&self) -> bool {
        self.is_local
    }

    pub fn set_is_local(&mut self, is_local: bool) {
        self.is_local = is_local;
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagramEdge {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<Uuid>,
    from: NodeId,
    to: NodeId,
    kind: RelationKind,
    label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_port: Option<PortSide>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target_port: Option<PortSide>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    directed: Option<bool>,
}

impl DiagramEdge {
    pub fn new(from: NodeId, to: NodeId, kind: RelationKind) -> Self {
        Self {
            id: Some(Uuid::new_v4()),
            from,
            to,
            kind,
            label: None,
            source_port: None,
            target_port: None,
            directed: if kind == RelationKind::Association {
                Some(true)
            } else {
                None
            },
        }
    }

    pub fn with_label(from: NodeId, to: NodeId, kind: RelationKind, label: Option<String>) -> Self {
        Self {
            id: Some(Uuid::new_v4()),
            from,
            to,
            kind,
            label,
            source_port: None,
            target_port: None,
            directed: if kind == RelationKind::Association {
                Some(true)
            } else {
                None
            },
        }
    }

    pub fn id(&self) -> Option<Uuid> {
        self.id
    }

    pub fn set_id(&mut self, id: Uuid) {
        self.id = Some(id);
    }

    pub fn with_ports(
        from: NodeId,
        to: NodeId,
        kind: RelationKind,
        label: Option<String>,
        source_port: Option<PortSide>,
        target_port: Option<PortSide>,
    ) -> Self {
        Self {
            id: Some(Uuid::new_v4()),
            from,
            to,
            kind,
            label,
            source_port,
            target_port,
            directed: if kind == RelationKind::Association {
                Some(true)
            } else {
                None
            },
        }
    }

    pub fn with_all(
        from: NodeId,
        to: NodeId,
        kind: RelationKind,
        label: Option<String>,
        source_port: Option<PortSide>,
        target_port: Option<PortSide>,
        directed: Option<bool>,
    ) -> Self {
        Self {
            id: Some(Uuid::new_v4()),
            from,
            to,
            kind,
            label,
            source_port,
            target_port,
            directed,
        }
    }

    pub fn from(&self) -> NodeId {
        self.from
    }

    pub fn to(&self) -> NodeId {
        self.to
    }

    pub fn kind(&self) -> RelationKind {
        self.kind
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub fn set_label(&mut self, label: Option<String>) {
        self.label = label;
    }

    pub fn set_kind(&mut self, kind: RelationKind) {
        self.kind = kind;
        if kind == RelationKind::Association && self.directed.is_none() {
            self.directed = Some(true);
        }
    }

    pub fn source_port(&self) -> Option<PortSide> {
        self.source_port
    }

    pub fn target_port(&self) -> Option<PortSide> {
        self.target_port
    }

    pub fn set_ports(&mut self, source_port: Option<PortSide>, target_port: Option<PortSide>) {
        self.source_port = source_port;
        self.target_port = target_port;
    }

    pub fn is_directed(&self) -> bool {
        if self.kind == RelationKind::Association {
            self.directed.unwrap_or(true)
        } else {
            false
        }
    }

    pub fn directed(&self) -> Option<bool> {
        self.directed
    }

    pub fn set_directed(&mut self, directed: bool) {
        self.directed = Some(directed);
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConceptGraph {
    nodes: Vec<DiagramNode>,
    edges: Vec<DiagramEdge>,
}

impl ConceptGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, concept: &Concept) -> NodeId {
        let node_count = self.nodes.len() as f32;
        let x = 60.0 + (node_count % 3.0) * 230.0;
        let y = 60.0 + (node_count / 3.0).floor() * 130.0;
        let node = DiagramNode::new(concept, x, y);
        let id = node.id();
        self.nodes.push(node);
        id
    }

    pub fn add_node_at(&mut self, concept: &Concept, x: f32, y: f32) -> NodeId {
        let snapped_x = (x / GRID_SIZE).round() * GRID_SIZE;
        let snapped_y = (y / GRID_SIZE).round() * GRID_SIZE;
        if let Some(existing) = self.find_node_by_concept_mut(concept.id()) {
            existing.set_position(snapped_x, snapped_y);
            return existing.id();
        }
        let node = DiagramNode::new(concept, snapped_x, snapped_y);
        let id = node.id();
        self.nodes.push(node);
        id
    }

    pub fn add_relation(&mut self, from: NodeId, to: NodeId, kind: RelationKind) {
        self.edges.push(DiagramEdge::new(from, to, kind));
    }

    pub fn add_relation_with_label(
        &mut self,
        from: NodeId,
        to: NodeId,
        kind: RelationKind,
        label: Option<String>,
    ) {
        self.edges
            .push(DiagramEdge::with_label(from, to, kind, label));
    }

    pub fn add_relation_full(
        &mut self,
        id: Uuid,
        from: NodeId,
        to: NodeId,
        kind: RelationKind,
        label: Option<String>,
    ) {
        let mut edge = DiagramEdge::with_label(from, to, kind, label);
        edge.set_id(id);
        self.edges.push(edge);
    }

    pub fn remove_relation_by_id(&mut self, id: Uuid) -> bool {
        let initial_len = self.edges.len();
        self.edges
            .retain(|e| e.id != Some(id) && e.from != id && e.to != id);
        self.edges.len() < initial_len
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn nodes(&self) -> &[DiagramNode] {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut Vec<DiagramNode> {
        &mut self.nodes
    }

    pub fn edges(&self) -> &[DiagramEdge] {
        &self.edges
    }

    pub fn edges_mut(&mut self) -> &mut Vec<DiagramEdge> {
        &mut self.edges
    }

    pub fn find_node(&self, id: NodeId) -> Option<&DiagramNode> {
        self.nodes.iter().find(|n| n.id() == id)
    }

    pub fn find_node_mut(&mut self, id: NodeId) -> Option<&mut DiagramNode> {
        self.nodes.iter_mut().find(|n| n.id() == id)
    }

    pub fn find_node_by_concept(&self, concept_id: Uuid) -> Option<&DiagramNode> {
        self.nodes.iter().find(|n| n.concept_id() == concept_id)
    }

    pub fn find_node_by_concept_mut(&mut self, concept_id: Uuid) -> Option<&mut DiagramNode> {
        self.nodes.iter_mut().find(|n| n.concept_id() == concept_id)
    }

    pub fn is_concept_on_diagram(&self, concept_id: Uuid) -> bool {
        self.find_node_by_concept(concept_id).is_some()
    }

    pub fn update_node_position(&mut self, id: NodeId, x: f32, y: f32) {
        if let Some(node) = self.find_node_mut(id) {
            let snapped_x = (x / GRID_SIZE).round() * GRID_SIZE;
            let snapped_y = (y / GRID_SIZE).round() * GRID_SIZE;
            node.set_position(snapped_x, snapped_y);
        }
    }

    pub fn remove_node(&mut self, id: NodeId) {
        if let Some(idx) = self.nodes.iter().position(|n| n.id() == id) {
            self.nodes.remove(idx);
        }
        // Kaskadesletning af tilknyttede relationer (Fail-Closed mod dangling edges)
        self.edges.retain(|e| e.from != id && e.to != id);
    }

    pub fn remove_node_by_concept(&mut self, concept_id: Uuid) {
        if let Some(node) = self.find_node_by_concept(concept_id) {
            let id = node.id();
            self.remove_node(id);
        }
    }

    pub fn remove_relation(&mut self, from: NodeId, to: NodeId) {
        self.edges.retain(|e| !(e.from == from && e.to == to));
    }

    pub fn find_edge(&self, from: NodeId, to: NodeId) -> Option<&DiagramEdge> {
        self.edges.iter().find(|e| e.from == from && e.to == to)
    }

    pub fn find_edge_mut(&mut self, from: NodeId, to: NodeId) -> Option<&mut DiagramEdge> {
        self.edges.iter_mut().find(|e| e.from == from && e.to == to)
    }

    pub fn update_edge_kind(&mut self, from: NodeId, to: NodeId, kind: RelationKind) -> bool {
        if let Some(edge) = self.find_edge_mut(from, to) {
            edge.set_kind(kind);
            true
        } else {
            false
        }
    }

    pub fn update_edge_label(&mut self, from: NodeId, to: NodeId, label: Option<String>) -> bool {
        if let Some(edge) = self.find_edge_mut(from, to) {
            edge.set_label(label);
            true
        } else {
            false
        }
    }

    pub fn update_edge_ports(
        &mut self,
        from: NodeId,
        to: NodeId,
        source_port: Option<PortSide>,
        target_port: Option<PortSide>,
    ) -> bool {
        if let Some(edge) = self.find_edge_mut(from, to) {
            edge.set_ports(source_port, target_port);
            true
        } else {
            false
        }
    }

    pub fn update_edge_directed(&mut self, from: NodeId, to: NodeId, directed: bool) -> bool {
        if let Some(edge) = self.find_edge_mut(from, to) {
            edge.set_directed(directed);
            true
        } else {
            false
        }
    }

    pub fn reverse_relation(&mut self, from: NodeId, to: NodeId) -> bool {
        if let Some(pos) = self.edges.iter().position(|e| e.from == from && e.to == to) {
            let mut edge = self.edges.remove(pos);
            let old_from = edge.from;
            let old_to = edge.to;
            let old_src_port = edge.source_port;
            let old_tgt_port = edge.target_port;

            edge.from = old_to;
            edge.to = old_from;
            edge.source_port = old_tgt_port;
            edge.target_port = old_src_port;

            self.edges.push(edge);
            true
        } else {
            false
        }
    }

    pub fn sync_with_concepts(&mut self, concepts: &[Concept]) {
        // 1. Fjern noder der ikke længere findes i begrebslisten
        let valid_concept_ids: Vec<Uuid> = concepts.iter().map(|c| c.id()).collect();
        let removed_node_ids: Vec<NodeId> = self
            .nodes
            .iter()
            .filter(|n| !valid_concept_ids.contains(&n.concept_id()))
            .map(|n| n.id())
            .collect();

        for id in removed_node_ids {
            self.remove_node(id);
        }

        // 2. Tilføj nye eller opdater eksisterende
        for concept in concepts {
            if let Some(node) = self.find_node_by_concept_mut(concept.id()) {
                node.set_label(concept.preferred_term().to_string());
                node.set_is_local(concept.belongs_to_domain().is_local());
                node.width = DEFAULT_NODE_WIDTH;
                node.height = DEFAULT_NODE_HEIGHT;
            } else {
                let count = self.nodes.len() as f32;
                let x = 60.0 + (count % 3.0) * 230.0;
                let y = 60.0 + (count / 3.0).floor() * 130.0;
                let mut node = DiagramNode::new(concept, x, y);
                node.set_is_local(concept.belongs_to_domain().is_local());
                self.nodes.push(node);
            }
        }
    }
}
