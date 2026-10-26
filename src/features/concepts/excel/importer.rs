use crate::features::concepts::excel::types::{
    ExcelError, ExcelImportSummary, RejectedRow, UpsertSummary,
};
use crate::features::concepts::{BelongsToDomain, Concept, ConceptEnglishFields};
use crate::features::diagnostics::{log_error, log_info, log_warn};
use crate::features::model::ModelProject;
use calamine::{open_workbook_auto_from_rs, Data, Range, Reader};
use std::io::Cursor;
use uuid::Uuid;

/// Indlæser og parser begreber fra en Excel-fil i hukommelsen (.xlsx, .xls)
pub fn parse_concepts_from_xlsx_bytes(bytes: &[u8]) -> Result<ExcelImportSummary, ExcelError> {
    log_info(
        "excel",
        &format!(
            "Starter parsing af Excel-dokument ({} bytes)...",
            bytes.len()
        ),
    );

    let cursor = Cursor::new(bytes);
    let mut workbook = open_workbook_auto_from_rs(cursor)?;

    let sheet_names = workbook.sheet_names().to_vec();
    if sheet_names.is_empty() {
        log_error("excel", "Ingen regnearksfaner fundet i Excel-filen");
        return Err(ExcelError::NoSheetsFound);
    }

    log_info(
        "excel",
        &format!(
            "Fandt {} fane(r) i arket: {:?}",
            sheet_names.len(),
            sheet_names
        ),
    );

    // Evaluer alle tilgængelige ark for at finde det mest relevante ark med databegreber
    let mut candidate_summaries = Vec::new();
    for name in &sheet_names {
        if let Ok(range) = workbook.worksheet_range(name) {
            let summary = parse_sheet_range(&range, name);
            candidate_summaries.push(summary);
        }
    }

    // Udvælg det bedste ark:
    // 1. Ark med gyldige begreber prioriteres altid over tomme ark.
    // 2. Hvis et ark har tosprogethed ("da+en") OG har begreber, vælges dette for maksimal information.
    // 3. Ellers det ark, der har flest gyldige begreber.
    // 4. Hvis ingen ark har begreber, vælges standard prioritering ("da+en", "begrebsliste", eller første ark).
    let chosen_summary = {
        let with_concepts: Vec<&ExcelImportSummary> = candidate_summaries
            .iter()
            .filter(|s| !s.valid_concepts.is_empty())
            .collect();

        if let Some(da_en_with_data) = with_concepts
            .iter()
            .find(|s| s.sheet_name.to_lowercase().contains("da+en"))
        {
            (*da_en_with_data).clone()
        } else if let Some(max_concepts) = with_concepts
            .into_iter()
            .max_by_key(|s| s.valid_concepts.len())
        {
            max_concepts.clone()
        } else {
            // Fallback når arket er tomt: bevar forventet standardark
            let default_name = sheet_names
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

            candidate_summaries
                .into_iter()
                .find(|s| s.sheet_name == default_name)
                .unwrap_or_else(|| ExcelImportSummary {
                    valid_concepts: Vec::new(),
                    rejected_rows: Vec::new(),
                    sheet_name: default_name,
                })
        }
    };

    log_info(
        "excel",
        &format!(
            "Valgte ark \"{}\": {} gyldige begreber, {} afviste række(r)",
            chosen_summary.sheet_name,
            chosen_summary.valid_concepts.len(),
            chosen_summary.rejected_rows.len()
        ),
    );

    for rej in &chosen_summary.rejected_rows {
        log_warn(
            "excel",
            &format!(
                "Række {} afvist: \"{}\" ({})",
                rej.row_index, rej.preferred_term, rej.reason
            ),
        );
    }

    Ok(chosen_summary)
}

fn parse_sheet_range(range: &Range<Data>, sheet_name: &str) -> ExcelImportSummary {
    let mut last_header_idx = None;
    let mut col_map = ColumnIndices::default();

    // Scan de første rækker (op til 6) for at identificere kolonnehoveder
    for (r_idx, row) in range.rows().enumerate().take(6) {
        let indices = detect_columns(row);
        if indices.has_primary_headers() {
            last_header_idx = Some(r_idx);
            col_map.merge_with(&indices);
        } else if col_map.has_primary_headers() {
            // Hvis vi allerede har fundet en gyldig primær header i en tidligere række,
            // og denne række IKKE har overskriftskarakter, stopper vi scanningen så datarrækker ikke forveksles.
            let has_any_header_match = indices.preferred_term.is_some()
                || indices.definition.is_some()
                || indices.accepted_term.is_some()
                || indices.source.is_some();
            if !has_any_header_match {
                break;
            }
        }
    }

    if !col_map.has_primary_headers() {
        return ExcelImportSummary {
            valid_concepts: Vec::new(),
            rejected_rows: Vec::new(),
            sheet_name: sheet_name.to_string(),
        };
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

        // Tjek om rækken mod forventning er en gentaget header-række (kræver at BÅDE term og def er overskriftsnavne)
        let is_header = {
            let p = pref_term.to_lowercase();
            let d = def.to_lowercase();
            (p == "foretrukken term"
                || p == "foretrukken term *"
                || p == "foretrukken term (da)"
                || p == "foretrukken term (da) *"
                || p == "preflabel (da)"
                || p == "preflabel"
                || p == "begrebsnavn"
                || p == "term"
                || p == "navn")
                && (d == "definition"
                    || d == "definition *"
                    || d == "definition (da)"
                    || d == "definition (da) *"
                    || d == "betydning"
                    || d == "forklaring")
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

    ExcelImportSummary {
        valid_concepts,
        rejected_rows,
        sheet_name: sheet_name.to_string(),
    }
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

    let summary = UpsertSummary { inserted, updated };
    log_info(
        "excel",
        &format!(
            "Excel upsert afsluttet: {} nye begreber tilføjet, {} eksisterende opdateret",
            summary.inserted, summary.updated
        ),
    );
    summary
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
        self.preferred_term.is_some() && self.definition.is_some()
    }

    fn merge_with(&mut self, other: &ColumnIndices) {
        if other.preferred_term.is_some() {
            self.preferred_term = other.preferred_term;
        }
        if other.definition.is_some() {
            self.definition = other.definition;
        }
        if other.accepted_term.is_some() {
            self.accepted_term = other.accepted_term;
        }
        if other.deprecated_term.is_some() {
            self.deprecated_term = other.deprecated_term;
        }
        if other.example.is_some() {
            self.example = other.example;
        }
        if other.comment.is_some() {
            self.comment = other.comment;
        }
        if other.application_note.is_some() {
            self.application_note = other.application_note;
        }
        if other.legal_source.is_some() {
            self.legal_source = other.legal_source;
        }
        if other.source.is_some() {
            self.source = other.source;
        }
        if other.domain.is_some() {
            self.domain = other.domain;
        }
        if other.identifier.is_some() {
            self.identifier = other.identifier;
        }
        if other.derived_from.is_some() {
            self.derived_from = other.derived_from;
        }
        if other.en_preferred_term.is_some() {
            self.en_preferred_term = other.en_preferred_term;
        }
        if other.en_accepted_term.is_some() {
            self.en_accepted_term = other.en_accepted_term;
        }
        if other.en_deprecated_term.is_some() {
            self.en_deprecated_term = other.en_deprecated_term;
        }
        if other.en_definition.is_some() {
            self.en_definition = other.en_definition;
        }
        if other.en_example.is_some() {
            self.en_example = other.en_example;
        }
        if other.en_comment.is_some() {
            self.en_comment = other.en_comment;
        }
        if other.en_application_note.is_some() {
            self.en_application_note = other.en_application_note;
        }
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
        if text.is_empty() || text.len() > 50 {
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
            } else if text == "definition (en)"
                || text.starts_with("definition (en)")
                || text == "definition"
            {
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
            || text.contains("begrebsnavn")
            || text.contains("termnavn")
        {
            cols.preferred_term = Some(idx);
        } else if text == "definition"
            || text.starts_with("definition ")
            || text.starts_with("definition*")
            || text.starts_with("definition (da)")
            || text == "definition (da)"
            || text == "betydning"
            || text == "forklaring"
            || text == "beskrivelse"
        {
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
