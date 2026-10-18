//! Modular UI components for FDA Information Model decomposed according to Package-by-Feature.

pub mod association_editor;
pub mod attribute_editor;
pub mod class_detail;
pub mod class_list;
pub mod enum_detail;

use crate::features::information_model::models::InformationDataType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTypeOption {
    pub label: String,
    pub data_type: InformationDataType,
}

impl std::fmt::Display for DataTypeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}
