use crate::features::concepts::excel::ExcelImportSummary;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ExcelImportPreviewState {
    pub import_summary: ExcelImportSummary,
    pub new_count: usize,
    pub update_count: usize,
    pub file_path: PathBuf,
}
