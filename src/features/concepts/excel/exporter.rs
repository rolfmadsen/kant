use crate::features::concepts::excel::types::ExcelError;
use crate::features::concepts::BelongsToDomain;
use crate::features::model::ModelProject;
use rust_xlsxwriter::{Format, Workbook};

/// Eksporterer hele modellens begrebsliste til en officiel FDA-arbejdsbog (.xlsx)
pub fn export_project_concepts_to_xlsx(project: &ModelProject) -> Result<Vec<u8>, ExcelError> {
    let mut workbook = Workbook::new();

    let bold_header_fmt = Format::new()
        .set_bold()
        .set_background_color(rust_xlsxwriter::Color::RGB(0xEE_F2_F6))
        .set_border(rust_xlsxwriter::FormatBorder::Thin);

    let skos_tag_fmt = Format::new()
        .set_italic()
        .set_font_color(rust_xlsxwriter::Color::RGB(0x64_74_8B))
        .set_background_color(rust_xlsxwriter::Color::RGB(0xF8_FA_FC));

    let data_fmt = Format::new()
        .set_text_wrap()
        .set_align(rust_xlsxwriter::FormatAlign::Top);

    // 1. Ark: Forretningsmetadata
    let meta_sheet = workbook.add_worksheet();
    meta_sheet.set_name("Forretningsmetadata")?;
    meta_sheet.set_column_width(0, 25)?;
    meta_sheet.set_column_width(1, 40)?;
    meta_sheet.set_column_width(2, 60)?;

    meta_sheet.write_with_format(0, 0, "Egenskab", &bold_header_fmt)?;
    meta_sheet.write_with_format(0, 1, "Værdi", &bold_header_fmt)?;
    meta_sheet.write_with_format(0, 2, "Beskrivelse", &bold_header_fmt)?;

    let meta = project.metadata();
    let meta_rows = [
        ("Modelnavn", meta.name(), "Modellens officielle navn"),
        (
            "Beskrivelse",
            meta.description(),
            "Kort beskrivelse af modellens formål",
        ),
        ("Namespace / URI", meta.uri(), "Basis HTTP-URI for modellen"),
        (
            "Modelansvarlig",
            meta.responsible_org(),
            "Organisation der står inde for modellen",
        ),
        ("Emne", meta.domain_area(), "Tematisk emnekategori"),
        ("Version", meta.version(), "Aktuelt versionsnummer"),
        (
            "Status",
            match meta.status() {
                crate::features::model::ModelStatus::Development => "Under udarbejdelse",
                crate::features::model::ModelStatus::Completed => "Gældende / Godkendt",
                crate::features::model::ModelStatus::Deprecated => "Udfaset",
                crate::features::model::ModelStatus::Withdrawn => "Tilbagetrukket",
            },
            "Modellens godkendelsesstatus",
        ),
        ("Modelsprog", "da", "Primært sprog for modellen"),
    ];

    for (idx, (prop, val, desc)) in meta_rows.iter().enumerate() {
        let r = (idx + 1) as u32;
        meta_sheet.write(r, 0, *prop)?;
        meta_sheet.write(r, 1, *val)?;
        meta_sheet.write(r, 2, *desc)?;
    }

    // 2. Ark: Begrebsliste DA
    let da_sheet = workbook.add_worksheet();
    da_sheet.set_name("Begrebsliste DA")?;

    let da_skos = [
        "prefLabel (da)",
        "altLabel (da)",
        "hiddenLabel (da)",
        "definition (da)",
        "example (da)",
        "comment (da)",
        "applicationNote",
        "legalSource",
        "source",
        "eget",
        "URI",
        "wasDerivedFrom",
    ];
    let da_headers = [
        "Foretrukken term *",
        "Accepteret term",
        "Frarådet term",
        "Definition *",
        "Eksempel",
        "Kommentar",
        "Anvendelsesnote",
        "Juridisk kilde",
        "Kilde",
        "Tilhører emne",
        "Identifikator",
        "Afledt af",
    ];

    da_sheet.set_column_width(0, 22)?;
    da_sheet.set_column_width(1, 18)?;
    da_sheet.set_column_width(2, 18)?;
    da_sheet.set_column_width(3, 40)?;
    da_sheet.set_column_width(4, 25)?;
    da_sheet.set_column_width(5, 25)?;
    da_sheet.set_column_width(6, 25)?;
    da_sheet.set_column_width(7, 22)?;
    da_sheet.set_column_width(8, 20)?;
    da_sheet.set_column_width(9, 14)?;
    da_sheet.set_column_width(10, 35)?;
    da_sheet.set_column_width(11, 35)?;

    for (c, tag) in da_skos.iter().enumerate() {
        da_sheet.write_with_format(0, c as u16, *tag, &skos_tag_fmt)?;
    }
    for (c, h) in da_headers.iter().enumerate() {
        da_sheet.write_with_format(1, c as u16, *h, &bold_header_fmt)?;
    }

    for (r_idx, c) in project.concepts().iter().enumerate() {
        let r = (r_idx + 2) as u32;
        let belongs_str = match c.belongs_to_domain() {
            BelongsToDomain::Yes => "Ja".to_string(),
            BelongsToDomain::No => "Nej".to_string(),
            BelongsToDomain::ModelRef(uri) => format!("Model: {}", uri),
        };

        da_sheet.write_with_format(r, 0, c.preferred_term(), &data_fmt)?;
        da_sheet.write_with_format(r, 1, c.accepted_term().unwrap_or(""), &data_fmt)?;
        da_sheet.write_with_format(r, 2, c.deprecated_term().unwrap_or(""), &data_fmt)?;
        da_sheet.write_with_format(r, 3, c.definition(), &data_fmt)?;
        da_sheet.write_with_format(r, 4, c.example().unwrap_or(""), &data_fmt)?;
        da_sheet.write_with_format(r, 5, c.comment().unwrap_or(""), &data_fmt)?;
        da_sheet.write_with_format(r, 6, c.application_note().unwrap_or(""), &data_fmt)?;
        da_sheet.write_with_format(r, 7, c.legal_source().unwrap_or(""), &data_fmt)?;
        da_sheet.write_with_format(r, 8, c.source().unwrap_or(""), &data_fmt)?;
        da_sheet.write_with_format(r, 9, &belongs_str, &data_fmt)?;
        da_sheet.write_with_format(r, 10, c.identifier().unwrap_or(""), &data_fmt)?;
        da_sheet.write_with_format(r, 11, c.derived_from().unwrap_or(""), &data_fmt)?;
    }

    // 3. Ark: Begrebsliste DA+EN
    let da_en_sheet = workbook.add_worksheet();
    da_en_sheet.set_name("Begrebsliste DA+EN")?;

    let da_en_skos = [
        "prefLabel (da)",
        "altLabel (da)",
        "hiddenLabel (da)",
        "definition (da)",
        "example (da)",
        "comment (da)",
        "applicationNote",
        "legalSource",
        "source",
        "eget",
        "URI",
        "wasDerivedFrom",
        "prefLabel (en)",
        "altLabel (en)",
        "hiddenLabel (en)",
        "definition (en)",
        "example (en)",
        "comment (en)",
        "applicationNote (en)",
    ];
    let da_en_headers = [
        "Foretrukken term (da) *",
        "Accepteret term (da)",
        "Frarådet term (da)",
        "Definition (da) *",
        "Eksempel (da)",
        "Kommentar (da)",
        "Anvendelsesnote (da)",
        "Juridisk kilde",
        "Kilde",
        "Tilhører emne",
        "Identifikator",
        "Afledt af",
        "Foretrukken term (en)",
        "Accepteret term (en)",
        "Frarådet term (en)",
        "Definition (en)",
        "Eksempel (en)",
        "Kommentar (en)",
        "Anvendelsesnote (en)",
    ];

    for c in 0..19 {
        da_en_sheet.set_column_width(c, if c == 3 || c == 15 { 38 } else { 22 })?;
    }

    for (c, tag) in da_en_skos.iter().enumerate() {
        da_en_sheet.write_with_format(0, c as u16, *tag, &skos_tag_fmt)?;
    }
    for (c, h) in da_en_headers.iter().enumerate() {
        da_en_sheet.write_with_format(1, c as u16, *h, &bold_header_fmt)?;
    }

    for (r_idx, c) in project.concepts().iter().enumerate() {
        let r = (r_idx + 2) as u32;
        let belongs_str = match c.belongs_to_domain() {
            BelongsToDomain::Yes => "Ja".to_string(),
            BelongsToDomain::No => "Nej".to_string(),
            BelongsToDomain::ModelRef(uri) => format!("Model: {}", uri),
        };

        da_en_sheet.write_with_format(r, 0, c.preferred_term(), &data_fmt)?;
        da_en_sheet.write_with_format(r, 1, c.accepted_term().unwrap_or(""), &data_fmt)?;
        da_en_sheet.write_with_format(r, 2, c.deprecated_term().unwrap_or(""), &data_fmt)?;
        da_en_sheet.write_with_format(r, 3, c.definition(), &data_fmt)?;
        da_en_sheet.write_with_format(r, 4, c.example().unwrap_or(""), &data_fmt)?;
        da_en_sheet.write_with_format(r, 5, c.comment().unwrap_or(""), &data_fmt)?;
        da_en_sheet.write_with_format(r, 6, c.application_note().unwrap_or(""), &data_fmt)?;
        da_en_sheet.write_with_format(r, 7, c.legal_source().unwrap_or(""), &data_fmt)?;
        da_en_sheet.write_with_format(r, 8, c.source().unwrap_or(""), &data_fmt)?;
        da_en_sheet.write_with_format(r, 9, &belongs_str, &data_fmt)?;
        da_en_sheet.write_with_format(r, 10, c.identifier().unwrap_or(""), &data_fmt)?;
        da_en_sheet.write_with_format(r, 11, c.derived_from().unwrap_or(""), &data_fmt)?;

        let en = c.english();
        da_en_sheet.write_with_format(
            r,
            12,
            en.and_then(|e| e.preferred_term.as_deref()).unwrap_or(""),
            &data_fmt,
        )?;
        da_en_sheet.write_with_format(
            r,
            13,
            en.and_then(|e| e.accepted_term.as_deref()).unwrap_or(""),
            &data_fmt,
        )?;
        da_en_sheet.write_with_format(
            r,
            14,
            en.and_then(|e| e.deprecated_term.as_deref()).unwrap_or(""),
            &data_fmt,
        )?;
        da_en_sheet.write_with_format(
            r,
            15,
            en.and_then(|e| e.definition.as_deref()).unwrap_or(""),
            &data_fmt,
        )?;
        da_en_sheet.write_with_format(
            r,
            16,
            en.and_then(|e| e.example.as_deref()).unwrap_or(""),
            &data_fmt,
        )?;
        da_en_sheet.write_with_format(
            r,
            17,
            en.and_then(|e| e.comment.as_deref()).unwrap_or(""),
            &data_fmt,
        )?;
        da_en_sheet.write_with_format(
            r,
            18,
            en.and_then(|e| e.application_note.as_deref()).unwrap_or(""),
            &data_fmt,
        )?;
    }

    let buffer = workbook.save_to_buffer()?;
    Ok(buffer)
}
