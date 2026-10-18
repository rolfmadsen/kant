//! Validation and integrity verification for FDA Information Model.
//!
//! Enforces structural invariants and FDA Modelregel 19 naming rules across
//! classes, attributes, multiplicities and relations.

use crate::features::information_model::linter::{NamingIssue, NamingLinter};
use crate::features::information_model::models::{
    InformationDataType, InformationModel,
};
use std::collections::HashSet;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ValidationError {
    #[error("Klasse med id {0} har et tomt navn")]
    EmptyClassName(Uuid),

    #[error("Dublet klassenavn: '{0}' findes mere end én gang")]
    DuplicateClassName(String),

    #[error("Klassenavn '{name}' overtræder FDA navnekonvention: {issue:?}")]
    InvalidClassNaming { name: String, issue: NamingIssue },

    #[error("Attribut på klasse '{class_name}' har et tomt navn")]
    EmptyAttributeName { class_name: String, attr_id: Uuid },

    #[error("Dublet attributnavn: '{attribute_name}' på klasse '{class_name}'")]
    DuplicateAttributeName {
        class_name: String,
        attribute_name: String,
    },

    #[error("Attributnavn '{attribute_name}' på klasse '{class_name}' overtræder FDA navnekonvention")]
    InvalidAttributeNaming {
        class_name: String,
        attribute_name: String,
        issue: NamingIssue,
    },

    #[error("Ugyldig multiplicitet: {details}")]
    InvalidMultiplicity { details: String },

    #[error("Relation refererer til en ikke-eksisterende klasse {missing_class_id}")]
    InvalidRelationEndpoint {
        relation_id: Uuid,
        missing_class_id: Uuid,
    },

    #[error("Attribut '{attribute_name}' på '{class_name}' refererer til ukendt enumeration {enumeration_id}")]
    MissingReferencedEnumeration {
        class_name: String,
        attribute_name: String,
        enumeration_id: Uuid,
    },

    #[error("Attribut '{attribute_name}' på '{class_name}' refererer til ukendt datatype {structured_id}")]
    MissingReferencedStructuredType {
        class_name: String,
        attribute_name: String,
        structured_id: Uuid,
    },
}

pub struct InformationModelValidator;

impl InformationModelValidator {
    /// Validerer hele informationsmodellen for strukturel integritet og FDA-regler.
    pub fn validate(model: &InformationModel) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        let mut class_names = HashSet::new();
        let class_ids: HashSet<Uuid> = model.classes().iter().map(|c| c.id()).collect();

        // 1. Valider klasser og deres attributter
        for class in model.classes() {
            let trimmed_name = class.name().trim();
            if trimmed_name.is_empty() {
                errors.push(ValidationError::EmptyClassName(class.id()));
            } else {
                let normalized = trimmed_name.to_lowercase();
                if !class_names.insert(normalized) {
                    errors.push(ValidationError::DuplicateClassName(trimmed_name.to_string()));
                }

                if let Some(issue) = NamingLinter::check_class_name(trimmed_name) {
                    errors.push(ValidationError::InvalidClassNaming {
                        name: trimmed_name.to_string(),
                        issue,
                    });
                }
            }

            let mut attr_names = HashSet::new();
            for attr in class.attributes() {
                let trimmed_attr = attr.name().trim();
                if trimmed_attr.is_empty() {
                    errors.push(ValidationError::EmptyAttributeName {
                        class_name: class.name().to_string(),
                        attr_id: attr.id(),
                    });
                } else {
                    let normalized_attr = trimmed_attr.to_lowercase();
                    if !attr_names.insert(normalized_attr) {
                        errors.push(ValidationError::DuplicateAttributeName {
                            class_name: class.name().to_string(),
                            attribute_name: trimmed_attr.to_string(),
                        });
                    }

                    if let Some(issue) = NamingLinter::check_attribute_name(trimmed_attr) {
                        errors.push(ValidationError::InvalidAttributeNaming {
                            class_name: class.name().to_string(),
                            attribute_name: trimmed_attr.to_string(),
                            issue,
                        });
                    }
                }

                // Tjek multiplicitet: lower <= upper hvis upper er Some
                let mult = attr.multiplicity();
                if let Some(upper) = mult.upper() {
                    if mult.lower() > upper {
                        errors.push(ValidationError::InvalidMultiplicity {
                            details: format!(
                                "Nedre grænse {} er større end øvre grænse {} for '{}.{}'",
                                mult.lower(),
                                upper,
                                class.name(),
                                attr.name()
                            ),
                        });
                    }
                }

                // Tjek referenced datatyper
                match attr.data_type() {
                    InformationDataType::Enumeration { enumeration_id } => {
                        if model.get_enumeration(*enumeration_id).is_none() {
                            errors.push(ValidationError::MissingReferencedEnumeration {
                                class_name: class.name().to_string(),
                                attribute_name: attr.name().to_string(),
                                enumeration_id: *enumeration_id,
                            });
                        }
                    }
                    InformationDataType::Structured { structured_id } => {
                        if model.get_structured_type(*structured_id).is_none() {
                            errors.push(ValidationError::MissingReferencedStructuredType {
                                class_name: class.name().to_string(),
                                attribute_name: attr.name().to_string(),
                                structured_id: *structured_id,
                            });
                        }
                    }
                    InformationDataType::Primitive(_) => {}
                }
            }
        }

        // 2. Valider relationer: endpoints skal eksistere
        for rel in model.relations() {
            if !class_ids.contains(&rel.from_class()) {
                errors.push(ValidationError::InvalidRelationEndpoint {
                    relation_id: rel.id(),
                    missing_class_id: rel.from_class(),
                });
            }
            if !class_ids.contains(&rel.to_class()) {
                // Kan referere til en enumeration eller structured type i dependency relationer
                let is_enum = model.get_enumeration(rel.to_class()).is_some();
                let is_struct = model.get_structured_type(rel.to_class()).is_some();
                if !is_enum && !is_struct {
                    errors.push(ValidationError::InvalidRelationEndpoint {
                        relation_id: rel.id(),
                        missing_class_id: rel.to_class(),
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl InformationModel {
    /// Kører strukturel validering på den aktuelle informationsmodel.
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        InformationModelValidator::validate(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::concept_model::RelationKind;
    use crate::features::information_model::models::{
        Attribute, ClassRelation, InformationClass, Multiplicity, PrimitiveType,
    };

    #[test]
    fn test_valid_model_passes_validation() {
        let mut model = InformationModel::new();
        let mut class = InformationClass::new("Kunde");
        class.add_attribute(Attribute::new(
            "kundenummer",
            PrimitiveType::Integer,
            Multiplicity::exactly_one(),
        ));
        model.add_class(class);

        assert!(model.validate().is_ok());
    }

    #[test]
    fn test_duplicate_class_name_fails() {
        let mut model = InformationModel::new();
        model.add_class(InformationClass::new("Kunde"));
        model.add_class(InformationClass::new("kunde"));

        let res = model.validate();
        assert!(res.is_err());
        let errors = res.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, ValidationError::DuplicateClassName(_))));
    }

    #[test]
    fn test_invalid_relation_endpoint_fails() {
        let mut model = InformationModel::new();
        let class = InformationClass::new("Kunde");
        let cid = model.add_class(class);

        let dangling_id = Uuid::new_v4();
        model.add_relation(ClassRelation::new(cid, dangling_id, RelationKind::Association, None));

        let res = model.validate();
        assert!(res.is_err());
        let errors = res.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, ValidationError::InvalidRelationEndpoint { .. })));
    }
}
