//! FDA Information Model domain module.
//!
//! Provides UML classes, attributes, multiplicities, relations, lineage tracking,
//! validation and diagram graph structures decomposed according to Package-by-Feature.

pub mod diagram;
pub mod lineage;
pub mod linter;
pub mod models;
pub mod ui;
pub mod validation;

pub use diagram::*;
pub use lineage::*;
pub use linter::{
    is_lower_camel_case, is_upper_camel_case, NamingConvention, NamingIssue, NamingIssueKind,
    NamingLinter, NamingSeverity, NamingTarget,
};
pub use models::*;
pub use validation::*;
