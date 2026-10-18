use crate::features::concepts::excel::types::{
    ExcelError, ExcelImportSummary, RejectedRow, UpsertSummary,
};
use crate::features::concepts::{BelongsToDomain, Concept, ConceptEnglishFields};
use crate::features::model::ModelProject;
use calamine::{open_workbook_auto_from_rs, Data, Reader};
use std::io::Cursor;
use uuid::Uuid;

/// Indlæser og parser begreber fra en Excel-fil i hukommelsen (.xlsx, .xls)
pub fn parse_concepts_from_xlsx_bytes(bytes: &[u8]) -> Result<ExcelImportSummary, ExcelError> {
    let cursor = Cursor::new(bytes);
    let mut workbook = open_workbook_auto_from_rs(cursor)?;

    let sheet_names = workbook.sheet_names().to_vec();
    if sheet_names.is_empty() {
        return Err(ExcelError::NoSheetsFound);
    }

    // Prioritering af arknavne: "DA+EN", dernæst "Begrebsliste", dernæst første ark
    let target_sheet = sheet_names
        .iter()
        .find(|name| name.to_lowercase().contains("da+en"))
        .or_else(|| {
            sheet_names
                .iter()
                .find(|name| name.to_lowercase().contains("begrebsliste"))
        })
        .or_else(|| sheet_names.first())
        .cloned()
        .unwrap();

    let range = workbook.worksheet_range(&target_sheet)?;

    let mut last_header_idx = None;
    let mut col_map = ColumnIndices::default();

    // Scan de første 6 rækker for at identificere kolonnehoveder (kan have flere header-rækker, fx SKOS i række 1 og danske navne i række 2 eller 3)
    for (r_idx, row) in range.rows().enumerate().take(6) {
        let indices = detect_columns(row);
        if indices.has_primary_headers() {
            last_header_idx = Some(r_idx);
            col_map = indices;
        }
    }

    // Start efter den senest detekterede header-række
    let start_row = last_header_idx.map(|idx| idx + 1).unwrap_or(1);

    let mut valid_concepts = Vec::new();
    let mut rejected_rows = Vec::new();

    for (r_idx, row) in range.rows().enumerate().skip(start_row) {
        let human_row = r_idx + 1;

        // Tjek om rækken er helt tom
        let has_content = row.iter().any(|cell| {
            let s = cell_as_string(cell);
            !s.trim().is_empty()
        });
        if !has_content {
            continue;
        }

        let pref_term = col_map
            .get_str(row, col_map.preferred_term)
            .trim()
            .to_string();
        let def = col_map.get_str(row, col_map.definition).trim().to_string();

        if pref_term.is_empty() && def.is_empty() {
            continue;
        }

        // Tjek om rækken mod forventning er en gentaget header-række
        let is_header = {
            let p = pref_term.to_lowercase();
            let d = def.to_lowercase();
            p.starts_with("foretrukken")
                || p.starts_with("preflabel")
                || p == "term"
                || p == "navn"
                || d.starts_with("definition")
        };
        if is_header {
            continue;
        }

        if pref_term.is_empty() {
            rejected_rows.push(RejectedRow {
                row_index: human_row,
                preferred_term: "(Ingen term)".to_string(),
                reason: "Mangler obligatorisk foretrukken term / navn".to_string(),
            });
            continue;
        }

        if def.is_empty() {
            rejected_rows.push(RejectedRow {
                row_index: human_row,
                preferred_term: pref_term,
                reason: "Mangler obligatorisk definition".to_string(),
            });
            continue;
        }

        let domain_str = col_map.get_str(row, col_map.domain);
        let belongs = BelongsToDomain::from_str_loose(&domain_str);

        let mut concept = Concept::new(&pref_term, &def, belongs);

        let opt = |s: String| {
            let t = s.trim().to_string();
            if t.is_empty() {
                None
            } else {
                Some(t)
            }
        };

        concept.set_accepted_term(opt(col_map.get_str(row, col_map.accepted_term)));
        concept.set_deprecated_term(opt(col_map.get_str(row, col_map.deprecated_term)));
        concept.set_example(opt(col_map.get_str(row, col_map.example)));
        concept.set_comment(opt(col_map.get_str(row, col_map.comment)));
        concept.set_application_note(opt(col_map.get_str(row, col_map.application_note)));
        concept.set_legal_source(opt(col_map.get_str(row, col_map.legal_source)));
        concept.set_source(opt(col_map.get_str(row, col_map.source)));
        concept.set_identifier(opt(col_map.get_str(row, col_map.identifier)));
        concept.set_derived_from(opt(col_map.get_str(row, col_map.derived_from)));

        // Engelsk sproglag
        let en_pref = opt(col_map.get_str(row, col_map.en_preferred_term));
        let en_acc = opt(col_map.get_str(row, col_map.en_accepted_term));
        let en_dep = opt(col_map.get_str(row, col_map.en_deprecated_term));
        let en_def = opt(col_map.get_str(row, col_map.en_definition));
        let en_ex = opt(col_map.get_str(row, col_map.en_example));
        let en_comm = opt(col_map.get_str(row, col_map.en_comment));
        let en_app = opt(col_map.get_str(row, col_map.en_application_note));

        let en_fields = ConceptEnglishFields {
            preferred_term: en_pref,
            accepted_term: en_acc,
            deprecated_term: en_dep,
            definition: en_def,
            example: en_ex,
            comment: en_comm,
            application_note: en_app,
        };

        if !en_fields.is_empty() {
            concept.set_english(Some(en_fields));
        }

        valid_concepts.push(concept);
    }

    Ok(ExcelImportSummary {
        valid_concepts,
        rejected_rows,
        sheet_name: target_sheet,
    })
}

/// Udfører intelligent upsert af importerede begreber på projektet:
/// Matcher på `identifier` (URI/ID) eller `preferred_term` (case-insensitive).
/// Bevarer eksisterende begrebers `Uuid` for at beskytte diagrammer og relationer.
pub fn apply_concept_upsert(project: &mut ModelProject, concepts: Vec<Concept>) -> UpsertSummary {
    let mut inserted = 0;
    let mut updated = 0;

    for imported in concepts {
        let existing_id = {
            let existing_list = project.concepts();

            // 1. Match på Identifikator (hvis ikke-tom)
            let id_match = imported.identifier().and_then(|imp_id| {
                let imp_trim = imp_id.trim();
                if imp_trim.is_empty() {
                    None
                } else {
                    existing_list
                        .iter()
                        .find(|c| {
                            c.identifier()
                                .map(|i| i.trim().eq_ignore_ascii_case(imp_trim))
                                .unwrap_or(false)
                        })
                        .map(|c| c.id())
                }
            });

            // 2. Fallback: Match på Foretrukken term (case-insensitiv)
            id_match.or_else(|| {
                let term_trim = imported.preferred_term().trim();
                existing_list
                    .iter()
                    .find(|c| c.preferred_term().trim().eq_ignore_ascii_case(term_trim))
                    .map(|c| c.id())
            })
        };

        if let Some(target_uuid) = existing_id {
            // Opdater eksisterende begreb, bevar UUID
            if let Some(existing) = project.get_concept_mut(target_uuid) {
                existing.set_definition(imported.definition());
                existing.set_belongs_to_domain(imported.belongs_to_domain().clone());

                if let Some(term) = imported.accepted_term() {
                    existing.set_accepted_term(Some(term.to_string()));
                }
                if let Some(term) = imported.deprecated_term() {
                    existing.set_deprecated_term(Some(term.to_string()));
                }
                if let Some(ex) = imported.example() {
                    existing.set_example(Some(ex.to_string()));
                }
                if let Some(comm) = imported.comment() {
                    existing.set_comment(Some(comm.to_string()));
                }
                if let Some(app) = imported.application_note() {
                    existing.set_application_note(Some(app.to_string()));
                }
                if let Some(leg) = imported.legal_source() {
                    existing.set_legal_source(Some(leg.to_string()));
                }
                if let Some(src) = imported.source() {
                    existing.set_source(Some(src.to_string()));
                }
                if let Some(id_uri) = imported.identifier() {
                    existing.set_identifier(Some(id_uri.to_string()));
                }
                if let Some(der) = imported.derived_from() {
                    existing.set_derived_from(Some(der.to_string()));
                }
                if let Some(en) = imported.english() {
                    existing.set_english(Some(en.clone()));
                }
                updated += 1;
            }
        } else {
            // Indsæt som nyt begreb
            let new_uuid = if imported.id().is_nil() {
                Uuid::new_v4()
            } else {
                imported.id()
            };
            let mut new_concept = Concept::new_with_id(
                new_uuid,
                imported.preferred_term(),
                imported.definition(),
                imported.belongs_to_domain().clone(),
            );
            new_concept.set_accepted_term(imported.accepted_term().map(str::to_string));
            new_concept.set_deprecated_term(imported.deprecated_term().map(str::to_string));
            new_concept.set_example(imported.example().map(str::to_string));
            new_concept.set_comment(imported.comment().map(str::to_string));
            new_concept.set_application_note(imported.application_note().map(str::to_string));
            new_concept.set_legal_source(imported.legal_source().map(str::to_string));
            new_concept.set_source(imported.source().map(str::to_string));
            new_concept.set_identifier(imported.identifier().map(str::to_string));
            new_concept.set_derived_from(imported.derived_from().map(str::to_string));
            new_concept.set_english(imported.english().cloned());

            let _ = project.add_concept(new_concept);
            inserted += 1;
        }
    }

    UpsertSummary { inserted, updated }
}

#[derive(Debug, Default, Clone)]
struct ColumnIndices {
    preferred_term: Option<usize>,
    definition: Option<usize>,
    accepted_term: Option<usize>,
    deprecated_term: Option<usize>,
    example: Option<usize>,
    comment: Option<usize>,
    application_note: Option<usize>,
    legal_source: Option<usize>,
    source: Option<usize>,
    domain: Option<usize>,
    identifier: Option<usize>,
    derived_from: Option<usize>,
    en_preferred_term: Option<usize>,
    en_accepted_term: Option<usize>,
    en_deprecated_term: Option<usize>,
    en_definition: Option<usize>,
    en_example: Option<usize>,
    en_comment: Option<usize>,
    en_application_note: Option<usize>,
}

impl ColumnIndices {
    fn has_primary_headers(&self) -> bool {
        self.preferred_term.is_some() || self.definition.is_some()
    }

    fn get_str(&self, row: &[Data], idx: Option<usize>) -> String {
        idx.and_then(|i| row.get(i))
            .map(cell_as_string)
            .unwrap_or_default()
    }
}

fn detect_columns(row: &[Data]) -> ColumnIndices {
    let mut cols = ColumnIndices::default();

    for (idx, cell) in row.iter().enumerate() {
        let text = cell_as_string(cell).trim().to_lowercase();
        if text.is_empty() {
            continue;
        }

        // Tjek engelske overskrifter først
        if text.contains("(en)") || text.contains("english") {
            if text.contains("altlabel") || text.contains("accepteret") || text.contains("accepted")
            {
                cols.en_accepted_term = Some(idx);
            } else if text.contains("hiddenlabel")
                || text.contains("frarådet")
                || text.contains("deprecated")
            {
                cols.en_deprecated_term = Some(idx);
            } else if text.contains("preflabel")
                || text.contains("foretrukken")
                || text.contains("preferred")
                || text == "term (en)"
                || text == "english term"
            {
                cols.en_preferred_term = Some(idx);
            } else if text.contains("definition") {
                cols.en_definition = Some(idx);
            } else if text.contains("example") || text.contains("eksempel") {
                cols.en_example = Some(idx);
            } else if text.contains("comment") || text.contains("kommentar") {
                cols.en_comment = Some(idx);
            } else if text.contains("application") || text.contains("anvendelse") {
                cols.en_application_note = Some(idx);
            }
            continue;
        }

        // Danske og generelle overskrifter
        if text.contains("altlabel") || text.contains("accepteret") || text.contains("godkendt") {
            cols.accepted_term = Some(idx);
        } else if text.contains("hiddenlabel") || text.contains("frarådet") {
            cols.deprecated_term = Some(idx);
        } else if text.contains("preflabel")
            || text.contains("foretrukken")
            || text == "navn"
            || text == "begreb"
            || text == "term"
        {
            cols.preferred_term = Some(idx);
        } else if text.contains("definition") || text == "betydning" || text == "forklaring" {
            cols.definition = Some(idx);
        } else if text.contains("example") || text.contains("eksempel") {
            cols.example = Some(idx);
        } else if text.contains("comment") || text.contains("kommentar") || text == "note" {
            cols.comment = Some(idx);
        } else if text.contains("anvendelsesnote") || text.contains("applicationnote") {
            cols.application_note = Some(idx);
        } else if text.contains("juridisk")
            || text.contains("legalsource")
            || text.contains("retskilde")
        {
            cols.legal_source = Some(idx);
        } else if text == "kilde" || text == "source" {
            cols.source = Some(idx);
        } else if text.contains("tilhører")
            || text.contains("emne")
            || text.contains("domæne")
            || text == "eget"
        {
            cols.domain = Some(idx);
        } else if text.contains("identifikator") || text == "uri" || text == "id" {
            cols.identifier = Some(idx);
        } else if text.contains("afledt")
            || text.contains("proveniens")
            || text.contains("wasderivedfrom")
        {
            cols.derived_from = Some(idx);
        }
    }

    cols
}

fn cell_as_string(cell: &Data) -> String {
    match cell {
        Data::String(s) => s.trim().to_string(),
        Data::Float(f) => {
            if f.fract() == 0.0 {
                format!("{:.0}", f)
            } else {
                format!("{}", f)
            }
        }
        Data::Int(i) => format!("{}", i),
        Data::Bool(b) => {
            if *b {
                "Ja".to_string()
            } else {
                "Nej".to_string()
            }
        }
        Data::DateTime(dt) => format!("{}", dt),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
        Data::Error(_) | Data::Empty => String::new(),
    }
}
