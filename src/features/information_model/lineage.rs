//! Concept lineage and traceability for FDA Information Model.
//!
//! Provides bidirectional traceability between terminological concepts (Begreber)
//! and structural information model elements (Klasser, Attributter) jf. FDA Modelreglerne.

use crate::features::concept_model::RelationKind;
use crate::features::concepts::Concept;
use crate::features::information_model::models::{
    Attribute, ClassRelation, InformationClass, InformationDataType, InformationModel,
};
use std::collections::HashSet;
use uuid::Uuid;

/// Repræsenterer en sporbarhedsrelation mellem et begreb og et informationselement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConceptLineageEntry {
    pub concept_id: Uuid,
    pub class_id: Uuid,
    pub class_name: String,
    pub attribute_id: Option<Uuid>,
    pub attribute_name: Option<String>,
}

impl InformationModel {
    /// Opretter en ny informationsklasse udledt direkte fra et FDA-begreb.
    pub fn create_class_from_concept(&mut self, concept: &Concept) -> Uuid {
        let class = InformationClass::from_concept(concept);
        self.add_class(class)
    }

    /// Finder alle informationsklasser, der er sporet/tilknyttet et givet begrebs-id.
    pub fn classes_for_concept(&self, concept_id: Uuid) -> Vec<&InformationClass> {
        self.classes()
            .iter()
            .filter(|c| c.concept_ids().contains(&concept_id))
            .collect()
    }

    /// Finder alle attributter på tværs af klasser, der er sporet til et givet begrebs-id.
    pub fn attributes_for_concept(&self, concept_id: Uuid) -> Vec<(&InformationClass, &Attribute)> {
        let mut results = Vec::new();
        for class in self.classes() {
            for attr in class.attributes() {
                if attr.concept_ids().contains(&concept_id) {
                    results.push((class, attr));
                }
            }
        }
        results
    }

    /// Fjerner samtlige referencer til et begrebs-id (både fra klasser og attributter).
    pub fn remove_concept_references(&mut self, concept_id: Uuid) {
        for class in self.classes_mut() {
            class.remove_concept_id(concept_id);
            for attr in class.attributes_mut() {
                attr.remove_concept_id(concept_id);
            }
        }
    }

    /// Synkroniserer afhængighedsrelationer for attributter med enumeration-datatyper.
    pub fn sync_attribute_dependencies(&mut self) {
        let mut desired_deps: HashSet<(Uuid, Uuid)> = HashSet::new();
        for class in self.classes() {
            for attr in class.attributes() {
                if let InformationDataType::Enumeration { enumeration_id } = attr.data_type() {
                    desired_deps.insert((class.id(), *enumeration_id));
                }
            }
        }
        for (from, to) in desired_deps {
            if self.find_relation(from, to).is_none() {
                self.relations_mut().push(ClassRelation::new(
                    from,
                    to,
                    RelationKind::Dependency,
                    Some("«use»".to_string()),
                ));
            }
        }
    }

    /// Genererer en samlet liste over alle lineage entries i modellen for et givet begreb.
    pub fn lineage_entries_for_concept(&self, concept_id: Uuid) -> Vec<ConceptLineageEntry> {
        let mut entries = Vec::new();
        for class in self.classes_for_concept(concept_id) {
            entries.push(ConceptLineageEntry {
                concept_id,
                class_id: class.id(),
                class_name: class.name().to_string(),
                attribute_id: None,
                attribute_name: None,
            });
        }
        for (class, attr) in self.attributes_for_concept(concept_id) {
            entries.push(ConceptLineageEntry {
                concept_id,
                class_id: class.id(),
                class_name: class.name().to_string(),
                attribute_id: Some(attr.id()),
                attribute_name: Some(attr.name().to_string()),
            });
        }
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::concepts::BelongsToDomain;
    use crate::features::information_model::models::{
        InformationEnumeration, Multiplicity, PrimitiveType,
    };

    #[test]
    fn test_lineage_queries_and_cleanup() {
        let mut model = InformationModel::new();
        let concept = Concept::new("Kunde", "Person der handler", BelongsToDomain::Yes);
        let cid = concept.id();

        let class_id = model.create_class_from_concept(&concept);
        assert_eq!(model.classes_for_concept(cid).len(), 1);

        let class = model.get_class_mut(class_id).unwrap();
        let mut attr = Attribute::new(
            "kundenummer",
            PrimitiveType::Integer,
            Multiplicity::exactly_one(),
        );
        attr.add_concept_id(cid);
        class.add_attribute(attr);

        assert_eq!(model.attributes_for_concept(cid).len(), 1);

        let entries = model.lineage_entries_for_concept(cid);
        assert_eq!(entries.len(), 2);

        model.remove_concept_references(cid);
        assert!(model.classes_for_concept(cid).is_empty());
        assert!(model.attributes_for_concept(cid).is_empty());
    }

    #[test]
    fn test_sync_attribute_dependencies() {
        let mut model = InformationModel::new();
        let enum_val = InformationEnumeration::new("KundeStatus", vec!["Aktiv".to_string()]);
        let enum_id = model.add_enumeration(enum_val);

        let mut class = InformationClass::new("Kunde");
        let attr = Attribute::new(
            "status",
            InformationDataType::Enumeration {
                enumeration_id: enum_id,
            },
            Multiplicity::exactly_one(),
        );
        class.add_attribute(attr);
        let class_id = model.add_class(class);

        assert!(model.relations().is_empty());
        model.sync_attribute_dependencies();

        assert_eq!(model.relations().len(), 1);
        let rel = &model.relations()[0];
        assert_eq!(rel.from_class(), class_id);
        assert_eq!(rel.to_class(), enum_id);
        assert_eq!(rel.kind(), RelationKind::Dependency);
        assert_eq!(rel.label(), Some("«use»"));
    }
}
