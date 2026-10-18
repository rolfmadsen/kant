use crate::features::concepts::Concept;
use thiserror::Error;

pub const FDA_TEMPLATE_BYTES: &[u8] =
    include_bytes!("../../../../docs/Begrebsliste_i_tabelformat_skabelon.xlsx");

/// Returnerer den indlejrede, officielle FDA Begrebslisteskabelon (.xlsx)
pub fn get_fda_template_xlsx_bytes() -> &'static [u8] {
    FDA_TEMPLATE_BYTES
}

#[derive(Debug, Error)]
pub enum ExcelError {
    #[error("Calamine regneark-fejl: {0}")]
    CalamineError(#[from] calamine::Error),

    #[error("XLSX skrivefejl: {0}")]
    XlsxWriterError(#[from] rust_xlsxwriter::XlsxError),

    #[error("Ingen gyldige regnearksfaner fundet i Excel-filen")]
    NoSheetsFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectedRow {
    pub row_index: usize,
    pub preferred_term: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct ExcelImportSummary {
    pub valid_concepts: Vec<Concept>,
    pub rejected_rows: Vec<RejectedRow>,
    pub sheet_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UpsertSummary {
    pub inserted: usize,
    pub updated: usize,
}

#[derive(Debug, Clone)]
pub struct SchemaGuideEntry {
    pub name: &'static str,
    pub tag: &'static str,
    pub required: bool,
    pub description: &'static str,
    pub example: &'static str,
}

pub fn get_fda_schema_guide_entries() -> Vec<SchemaGuideEntry> {
    vec![
        SchemaGuideEntry {
            name: "Foretrukken term",
            tag: "prefLabel (da)",
            required: true,
            description: "Begrebets officielle navn på dansk i naturligt sprog.",
            example: "Køretøj",
        },
        SchemaGuideEntry {
            name: "Definition",
            tag: "definition (da)",
            required: true,
            description: "Struktureret definition formuleret iht. Aristoteles' formel (overbegreb + adskillende egenskaber).",
            example: "Et transportmiddel på hjul til befordring af personer eller gods.",
        },
        SchemaGuideEntry {
            name: "Accepteret term",
            tag: "altLabel (da)",
            required: false,
            description: "Synonym eller tilladt betegnelse.",
            example: "Vogn",
        },
        SchemaGuideEntry {
            name: "Frarådet term",
            tag: "hiddenLabel (da)",
            required: false,
            description: "Uønsket, forældet eller misvisende betegnelse.",
            example: "Kærre",
        },
        SchemaGuideEntry {
            name: "Eksempel",
            tag: "example (da)",
            required: false,
            description: "Konkret forekomst der illustrerer begrebets anvendelse.",
            example: "En personbil eller en lastbil.",
        },
        SchemaGuideEntry {
            name: "Kommentar",
            tag: "comment (da)",
            required: false,
            description: "Supplerende bemærkning eller oplysning vedrørende begrebet.",
            example: "Omfatter ikke skinnebårne transportmidler som tog.",
        },
        SchemaGuideEntry {
            name: "Anvendelsesnote",
            tag: "applicationNote",
            required: false,
            description: "Note der beskriver anvendelse i en bestemt system- eller forretningskontekst.",
            example: "Anvendes ved registrering i Køretøjsregisteret.",
        },
        SchemaGuideEntry {
            name: "Juridisk kilde",
            tag: "legalSource",
            required: false,
            description: "Reference til lovgrundlag hvorfra begrebet udspringer.",
            example: "Færdselsloven § 2, nr. 12",
        },
        SchemaGuideEntry {
            name: "Kilde",
            tag: "source",
            required: false,
            description: "Reference til standard, fagordbog eller anden ressource.",
            example: "Dansk Standard DS 2400",
        },
        SchemaGuideEntry {
            name: "Tilhører emne",
            tag: "eget",
            required: false,
            description: "'Ja' (lokalt begreb), 'Nej' (indlånt begreb), eller model-URI.",
            example: "Ja",
        },
        SchemaGuideEntry {
            name: "Identifikator",
            tag: "URI",
            required: false,
            description: "Begrebets persistente HTTP-URI (genereres automatisk hvis udeladt).",
            example: "https://data.gov.dk/model/core/transport/Vehicle",
        },
        SchemaGuideEntry {
            name: "Afledt af",
            tag: "wasDerivedFrom",
            required: false,
            description: "Reference til oprindeligt begreb, hvis dette begreb er videreudviklet.",
            example: "https://schema.org/Vehicle",
        },
        SchemaGuideEntry {
            name: "Foretrukken term (en)",
            tag: "prefLabel (en)",
            required: false,
            description: "Engelsk oversættelse af foretrukken term (DA+EN ark).",
            example: "Vehicle",
        },
        SchemaGuideEntry {
            name: "Definition (en)",
            tag: "definition (en)",
            required: false,
            description: "Engelsk definition til international interoperabilitet (DA+EN ark).",
            example: "A means of transportation on wheels.",
        },
    ]
}
