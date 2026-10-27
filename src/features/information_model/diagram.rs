use crate::features::concept_model::{BendPoint, NodeId, PortSide, RelationKind, GRID_SIZE};
use crate::features::information_model::models::{ClassRelation, InformationModel, Multiplicity};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const DEFAULT_CLASS_NODE_WIDTH: f32 = 220.0;
pub const MIN_CLASS_NODE_HEIGHT: f32 = 80.0;
pub const ATTR_LINE_HEIGHT: f32 = 20.0;

pub fn calculate_class_node_height(attr_count: usize) -> f32 {
    let header_height = 54.0;
    let compartment_padding = 16.0;
    let attrs_height = (attr_count as f32) * ATTR_LINE_HEIGHT;
    let raw_height =
        (header_height + compartment_padding + attrs_height).max(MIN_CLASS_NODE_HEIGHT);
    (raw_height / GRID_SIZE).ceil() * GRID_SIZE
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassDiagramNode {
    id: NodeId,
    class_id: Uuid,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl ClassDiagramNode {
    pub fn new(class_id: Uuid, x: f32, y: f32, attr_count: usize) -> Self {
        Self {
            id: class_id,
            class_id,
            x,
            y,
            width: DEFAULT_CLASS_NODE_WIDTH,
            height: calculate_class_node_height(attr_count),
        }
    }

    pub fn id(&self) -> NodeId {
        self.id
    }
    pub fn class_id(&self) -> Uuid {
        self.class_id
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
        (self.height / GRID_SIZE).ceil() * GRID_SIZE
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn update_dimensions(&mut self, attr_count: usize) {
        self.height = calculate_class_node_height(attr_count);
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height() / 2.0)
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassDiagramEdge {
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_multiplicity: Option<Multiplicity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target_multiplicity: Option<Multiplicity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bends: Option<Vec<BendPoint>>,
}

impl ClassDiagramEdge {
    pub fn new(from: NodeId, to: NodeId, kind: RelationKind, label: Option<String>) -> Self {
        Self {
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
            source_multiplicity: None,
            target_multiplicity: None,
            bends: None,
        }
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
            source_multiplicity: None,
            target_multiplicity: None,
            bends: None,
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
            from,
            to,
            kind,
            label,
            source_port,
            target_port,
            directed,
            source_multiplicity: None,
            target_multiplicity: None,
            bends: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_multiplicities(
        from: NodeId,
        to: NodeId,
        kind: RelationKind,
        label: Option<String>,
        source_port: Option<PortSide>,
        target_port: Option<PortSide>,
        directed: Option<bool>,
        source_multiplicity: Option<Multiplicity>,
        target_multiplicity: Option<Multiplicity>,
    ) -> Self {
        Self {
            from,
            to,
            kind,
            label,
            source_port,
            target_port,
            directed,
            source_multiplicity,
            target_multiplicity,
            bends: None,
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
    pub fn set_kind(&mut self, kind: RelationKind) {
        self.kind = kind;
        if kind == RelationKind::Association && self.directed.is_none() {
            self.directed = Some(true);
        }
    }
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    pub fn set_label(&mut self, label: Option<String>) {
        self.label = label;
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
    pub fn source_multiplicity(&self) -> Option<Multiplicity> {
        self.source_multiplicity
    }
    pub fn set_source_multiplicity(&mut self, mult: Option<Multiplicity>) {
        self.source_multiplicity = mult;
    }
    pub fn target_multiplicity(&self) -> Option<Multiplicity> {
        self.target_multiplicity
    }
    pub fn set_target_multiplicity(&mut self, mult: Option<Multiplicity>) {
        self.target_multiplicity = mult;
    }
    pub fn bends(&self) -> Option<&[BendPoint]> {
        self.bends.as_deref()
    }
    pub fn set_bends(&mut self, bends: Option<Vec<BendPoint>>) {
        self.bends = bends;
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassGraph {
    nodes: Vec<ClassDiagramNode>,
    edges: Vec<ClassDiagramEdge>,
}

impl ClassGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn nodes(&self) -> &[ClassDiagramNode] {
        &self.nodes
    }
    pub fn nodes_mut(&mut self) -> &mut Vec<ClassDiagramNode> {
        &mut self.nodes
    }
    pub fn edges(&self) -> &[ClassDiagramEdge] {
        &self.edges
    }
    pub fn edges_mut(&mut self) -> &mut Vec<ClassDiagramEdge> {
        &mut self.edges
    }
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
    pub fn find_node(&self, id: NodeId) -> Option<&ClassDiagramNode> {
        self.nodes.iter().find(|n| n.id() == id)
    }
    pub fn find_node_mut(&mut self, id: NodeId) -> Option<&mut ClassDiagramNode> {
        self.nodes.iter_mut().find(|n| n.id() == id)
    }
    pub fn find_node_by_class(&self, class_id: Uuid) -> Option<&ClassDiagramNode> {
        self.nodes.iter().find(|n| n.class_id() == class_id)
    }
    pub fn find_node_by_class_mut(&mut self, class_id: Uuid) -> Option<&mut ClassDiagramNode> {
        self.nodes.iter_mut().find(|n| n.class_id() == class_id)
    }
    pub fn is_class_on_diagram(&self, class_id: Uuid) -> bool {
        self.nodes.iter().any(|n| n.class_id() == class_id)
    }

    pub fn add_node(&mut self, class_id: Uuid, attr_count: usize) -> NodeId {
        if let Some(existing) = self.find_node_by_class(class_id) {
            return existing.id();
        }
        let node_count = self.nodes.len() as f32;
        let x = 60.0 + (node_count % 3.0) * 260.0;
        let y = 60.0 + (node_count / 3.0).floor() * 160.0;
        let node = ClassDiagramNode::new(class_id, x, y, attr_count);
        let id = node.id();
        self.nodes.push(node);
        id
    }

    pub fn add_node_at(&mut self, class_id: Uuid, x: f32, y: f32, attr_count: usize) -> NodeId {
        if let Some(existing) = self.find_node_by_class_mut(class_id) {
            existing.set_position(x, y);
            return existing.id();
        }
        let node = ClassDiagramNode::new(class_id, x, y, attr_count);
        let id = node.id();
        self.nodes.push(node);
        id
    }

    pub fn remove_node(&mut self, node_id: NodeId) {
        self.nodes.retain(|n| n.id() != node_id);
        self.edges
            .retain(|e| e.from() != node_id && e.to() != node_id);
    }

    pub fn remove_class_node(&mut self, class_id: Uuid) {
        if let Some(node) = self.find_node_by_class(class_id) {
            let node_id = node.id();
            self.remove_node(node_id);
        }
    }

    pub fn add_relation(
        &mut self,
        from: NodeId,
        to: NodeId,
        kind: RelationKind,
        label: Option<String>,
    ) {
        self.add_relation_with_multiplicities(from, to, kind, label, None, None);
    }

    pub fn add_relation_with_multiplicities(
        &mut self,
        from: NodeId,
        to: NodeId,
        kind: RelationKind,
        label: Option<String>,
        source_multiplicity: Option<Multiplicity>,
        target_multiplicity: Option<Multiplicity>,
    ) {
        if from != to
            && self.find_node(from).is_some()
            && self.find_node(to).is_some()
            && !self
                .edges
                .iter()
                .any(|e| e.from() == from && e.to() == to && e.kind() == kind)
        {
            let mut edge = ClassDiagramEdge::new(from, to, kind, label);
            edge.set_source_multiplicity(source_multiplicity);
            edge.set_target_multiplicity(target_multiplicity);
            self.edges.push(edge);
        }
    }

    pub fn remove_relation(&mut self, from: NodeId, to: NodeId) {
        self.edges.retain(|e| !(e.from() == from && e.to() == to));
    }

    pub fn find_edge(&self, from: NodeId, to: NodeId) -> Option<&ClassDiagramEdge> {
        self.edges.iter().find(|e| e.from() == from && e.to() == to)
    }

    pub fn find_edge_mut(&mut self, from: NodeId, to: NodeId) -> Option<&mut ClassDiagramEdge> {
        self.edges
            .iter_mut()
            .find(|e| e.from() == from && e.to() == to)
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

    pub fn update_edge_bends(
        &mut self,
        from: NodeId,
        to: NodeId,
        bends: Option<Vec<BendPoint>>,
    ) -> bool {
        if let Some(edge) = self.find_edge_mut(from, to) {
            edge.set_bends(bends);
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

    pub fn update_edge_source_multiplicity(
        &mut self,
        from: NodeId,
        to: NodeId,
        mult: Option<Multiplicity>,
    ) -> bool {
        if let Some(edge) = self.find_edge_mut(from, to) {
            edge.set_source_multiplicity(mult);
            true
        } else {
            false
        }
    }

    pub fn update_edge_target_multiplicity(
        &mut self,
        from: NodeId,
        to: NodeId,
        mult: Option<Multiplicity>,
    ) -> bool {
        if let Some(edge) = self.find_edge_mut(from, to) {
            edge.set_target_multiplicity(mult);
            true
        } else {
            false
        }
    }

    pub fn reverse_relation(&mut self, from: NodeId, to: NodeId) -> bool {
        if let Some(pos) = self
            .edges
            .iter()
            .position(|e| e.from() == from && e.to() == to)
        {
            let mut edge = self.edges.remove(pos);
            let old_from = edge.from;
            let old_to = edge.to;
            let old_src_port = edge.source_port;
            let old_tgt_port = edge.target_port;
            let old_src_mult = edge.source_multiplicity;
            let old_tgt_mult = edge.target_multiplicity;

            edge.from = old_to;
            edge.to = old_from;
            edge.source_port = old_tgt_port;
            edge.target_port = old_src_port;
            edge.source_multiplicity = old_tgt_mult;
            edge.target_multiplicity = old_src_mult;

            self.edges.push(edge);
            true
        } else {
            false
        }
    }

    pub fn update_node_position(&mut self, id: NodeId, x: f32, y: f32) {
        if let Some(node) = self.find_node_mut(id) {
            node.set_position(x, y);
        }
    }

    pub fn update_class_dimensions(&mut self, class_id: Uuid, attr_count: usize) {
        if let Some(node) = self.find_node_by_class_mut(class_id) {
            node.update_dimensions(attr_count);
        }
    }

    pub fn sync_with_information_model(&mut self, info_model: &InformationModel) {
        let mut valid_entity_ids: Vec<Uuid> = info_model.classes().iter().map(|c| c.id()).collect();
        valid_entity_ids.extend(info_model.enumerations().iter().map(|e| e.id()));
        valid_entity_ids.extend(info_model.structured_types().iter().map(|st| st.id()));
        self.nodes
            .retain(|n| valid_entity_ids.contains(&n.class_id()));

        for node in &mut self.nodes {
            if node.id != node.class_id {
                let old_id = node.id;
                let new_id = node.class_id;
                node.id = new_id;
                for edge in &mut self.edges {
                    if edge.from == old_id {
                        edge.from = new_id;
                    }
                    if edge.to == old_id {
                        edge.to = new_id;
                    }
                }
            }
        }

        for node in &mut self.nodes {
            if let Some(class) = info_model.get_class(node.class_id()) {
                node.update_dimensions(class.attributes().len());
            } else if let Some(e) = info_model.get_enumeration(node.class_id()) {
                node.update_dimensions(e.values().len());
            } else if let Some(st) = info_model.get_structured_type(node.class_id()) {
                node.update_dimensions(st.attributes().len());
            }
        }

        let mut expected_edges: Vec<ClassDiagramEdge> = Vec::new();
        for rel in info_model.relations() {
            if let (Some(from_node), Some(to_node)) = (
                self.find_node_by_class(rel.from_class()),
                self.find_node_by_class(rel.to_class()),
            ) {
                let from_id = from_node.id();
                let to_id = to_node.id();
                if let Some(existing) = self.find_edge(from_id, to_id) {
                    let mut updated = existing.clone();
                    updated.set_kind(rel.kind());
                    updated.set_label(rel.label().map(String::from));
                    if rel.source_port().is_some() || rel.target_port().is_some() {
                        updated.set_ports(rel.source_port(), rel.target_port());
                    }
                    if let Some(d) = rel.directed() {
                        updated.set_directed(d);
                    }
                    updated.set_source_multiplicity(rel.source_multiplicity());
                    updated.set_target_multiplicity(rel.target_multiplicity());
                    if rel.bends().is_some() {
                        updated.set_bends(rel.bends().map(|b| b.to_vec()));
                    }
                    expected_edges.push(updated);
                } else {
                    let mut edge = ClassDiagramEdge::with_multiplicities(
                        from_id,
                        to_id,
                        rel.kind(),
                        rel.label().map(String::from),
                        rel.source_port(),
                        rel.target_port(),
                        rel.directed(),
                        rel.source_multiplicity(),
                        rel.target_multiplicity(),
                    );
                    if let Some(b) = rel.bends() {
                        edge.set_bends(Some(b.to_vec()));
                    }
                    expected_edges.push(edge);
                }
            }
        }

        if info_model.relations().is_empty() && !self.edges.is_empty() {
            let valid_node_ids: std::collections::HashSet<NodeId> =
                self.nodes.iter().map(|n| n.id()).collect();
            self.edges
                .retain(|e| valid_node_ids.contains(&e.from()) && valid_node_ids.contains(&e.to()));
        } else {
            self.edges = expected_edges;
        }
    }

    pub fn migrate_edges_to_model(&self, info_model: &mut InformationModel) {
        if info_model.relations().is_empty() && !self.edges.is_empty() {
            for edge in &self.edges {
                let from_class = self
                    .find_node(edge.from())
                    .map(|n| n.class_id())
                    .unwrap_or(edge.from());
                let to_class = self
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
                info_model.add_relation(rel);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_graph_lifecycle_and_cascading_removal() {
        let mut graph = ClassGraph::new();
        let class_1 = Uuid::new_v4();
        let class_2 = Uuid::new_v4();

        let n1 = graph.add_node(class_1, 0);
        let n2 = graph.add_node(class_2, 2);

        assert_eq!(graph.node_count(), 2);
        assert!(graph.is_class_on_diagram(class_1));
        assert!(graph.is_class_on_diagram(class_2));

        let node_1 = graph.find_node(n1).unwrap();
        let node_2 = graph.find_node(n2).unwrap();
        assert_eq!(node_1.height(), MIN_CLASS_NODE_HEIGHT);
        assert!(node_2.height() > node_1.height());

        graph.add_relation(n2, n1, RelationKind::Generalization, None);
        assert_eq!(graph.edge_count(), 1);

        graph.remove_class_node(class_1);
        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_class_node_height_snaps_to_grid_increments() {
        for attr_count in 0..10 {
            let h = calculate_class_node_height(attr_count);
            assert_eq!((h % GRID_SIZE).abs(), 0.0);
            if attr_count > 0 {
                let prev_h = calculate_class_node_height(attr_count - 1);
                assert!(h >= prev_h);
            }
        }
    }
}
