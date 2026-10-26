use kant::features::concept_model::RelationKind;
use kant::features::concepts::{BelongsToDomain, Concept};
use kant::features::export::{
    export_concept_model_svg, export_concepts_to_csv, export_information_model_svg,
    export_model_report_html, export_model_report_markdown, export_to_shacl_turtle,
    export_to_skos_turtle, export_to_xmi_2_1,
};
use kant::features::information_model::{
    Attribute, InformationClass, InformationDataType, InformationEnumeration, Multiplicity,
    PrimitiveType,
};
use kant::features::model::{ApprovalStatus, ModelMetadata, ModelProject, ModelStatus};
use kant::ui::app::{App, Message};

#[test]
fn test_task_059_export_engine_svg_csv_and_report() {
    // 1. Byg modelprojekt med metadata jf. Tabel D
    let mut metadata = ModelMetadata::new(
        "Køretøjsmodellen",
        "Kernemodel for registrering af motorkøretøjer i Danmark.",
        "https://data.gov.dk/model/core/vehicle",
        "Motorstyrelsen",
        "Transport og Trafik",
        "1.2.0",
        ModelStatus::Development,
    );
    metadata.set_legal_sources(vec![
        "Færdselsloven § 2, stk. 1".to_string(),
        "Bekendtgørelse om registrering af køretøjer § 10".to_string(),
    ]);
    metadata.set_version_notes(Some(
        "Opdateret med enumeration for drivkraft og SVG/CSV eksportstøtte.".to_string(),
    ));

    let mut project = ModelProject::new(metadata);

    // 2. Opret 12-kolonners begreber jf. Bilag D & E
    let mut c1 = Concept::new(
        "Køretøj",
        "Et mobilt teknisk anlæg, der anvendes til transport af personer eller gods.",
        BelongsToDomain::Yes,
    );
    c1.set_accepted_term(Some("Transportmiddel".to_string()));
    c1.set_deprecated_term(Some("Vogn".to_string()));
    c1.set_example(Some("Bil, cykel, traktor".to_string()));
    c1.set_comment(Some("Grundlæggende begreb for vejtransport.".to_string()));
    c1.set_application_note(Some("Anvendes i Motorregistret.".to_string()));
    c1.set_legal_source(Some("Færdselsloven § 2, stk. 1".to_string()));
    c1.set_source(Some("Retsinformation".to_string()));
    c1.set_identifier(Some(
        "https://data.gov.dk/model/core/vehicle/Koeretoej".to_string(),
    ));
    c1.set_derived_from(Some("https://schema.org/Vehicle".to_string()));

    let mut c2 = Concept::new(
        "Personbil",
        "Køretøj indrettet til befordring af højst 9 personer inklusive føreren.",
        BelongsToDomain::Yes,
    );
    c2.set_example(Some("Sedan, stationcar".to_string()));
    c2.set_identifier(Some(
        "https://data.gov.dk/model/core/vehicle/Personbil".to_string(),
    ));

    let mut c3 = Concept::new(
        "Registreringsattest",
        "Officielt dokument, der attesterer et køretøjs registrering.",
        BelongsToDomain::No,
    );
    // RFC-4180 test med citationstegn og kommaer i kommentaren
    c3.set_comment(Some(
        "Bemærk: Dokumentet indeholder \"stelnummer\", modelkode og ejerdata.".to_string(),
    ));

    project.concepts_mut().push(c1.clone());
    project.concepts_mut().push(c2.clone());
    project.concepts_mut().push(c3.clone());

    // 3. Tilføj noder og relationer i begrebsgrafen
    let n1 = project.concept_graph_mut().add_node(&c1);
    let n2 = project.concept_graph_mut().add_node(&c2);
    project
        .concept_graph_mut()
        .add_relation(n2, n1, RelationKind::Generalization);

    // 4. Byg Informationsmodel med klasser, attributter og enumerationer
    let enum_id = uuid::Uuid::new_v4();
    let fuel_enum = InformationEnumeration::new_with_id(
        enum_id,
        "DrivkraftType",
        vec![
            "benzin".to_string(),
            "diesel".to_string(),
            "el".to_string(),
            "brint".to_string(),
        ],
    );
    project.information_model_mut().add_enumeration(fuel_enum);

    let mut personbil_class = InformationClass::from_concept(&c2);
    personbil_class.add_attribute(Attribute::new(
        "stelnummer",
        PrimitiveType::CharacterString,
        Multiplicity::exactly_one(),
    ));
    personbil_class.add_attribute(Attribute::new(
        "drivkraft",
        InformationDataType::Enumeration {
            enumeration_id: enum_id,
        },
        Multiplicity::zero_or_one(),
    ));
    let class_id = personbil_class.id();
    project.information_model_mut().add_class(personbil_class);
    project
        .information_graph_mut()
        .add_node_at(class_id, 100.0, 100.0, 2);

    // === AC1: Standardiseret Vektor-SVG Eksport ===
    let concept_svg = export_concept_model_svg(project.concept_graph(), project.concepts());
    assert!(
        concept_svg.starts_with("<svg "),
        "Begrebsmodel SVG skal starte med <svg"
    );
    assert!(
        concept_svg.ends_with("</svg>"),
        "Begrebsmodel SVG skal slutte med </svg>"
    );
    assert!(
        concept_svg.contains("xmlns=\"http://www.w3.org/2000/svg\""),
        "SVG skal have XML namespace"
    );
    assert!(
        concept_svg.contains("viewBox="),
        "SVG skal have afpasset viewBox"
    );
    assert!(
        concept_svg.contains("Køretøj"),
        "SVG skal indeholde nodetekst 'Køretøj'"
    );
    assert!(
        concept_svg.contains("Personbil"),
        "SVG skal indeholde nodetekst 'Personbil'"
    );
    // SVG Generaliseringspil (lukket hvid trekant)
    assert!(
        concept_svg.contains("<polygon") || concept_svg.contains("<path"),
        "SVG skal indeholde relationer"
    );

    let info_svg =
        export_information_model_svg(project.information_graph(), project.information_model());
    assert!(
        info_svg.starts_with("<svg "),
        "Informationsmodel SVG skal starte med <svg"
    );
    assert!(
        info_svg.ends_with("</svg>"),
        "Informationsmodel SVG skal slutte med </svg>"
    );
    assert!(
        info_svg.contains("Personbil"),
        "Informationsmodel SVG skal have klassen Personbil"
    );
    assert!(
        info_svg.contains("stelnummer"),
        "Informationsmodel SVG skal have attributten stelnummer"
    );
    assert!(
        info_svg.contains("«enumeration»"),
        "Informationsmodel SVG skal have «enumeration» stereotype"
    );
    assert!(
        info_svg.contains("DrivkraftType"),
        "Informationsmodel SVG skal vise DrivkraftType"
    );

    // === AC2: RFC-4180 CSV-eksport af begrebsliste med UTF-8 BOM ===
    let csv_data = export_concepts_to_csv(project.concepts());
    assert!(
        csv_data.starts_with('\u{FEFF}'),
        "CSV skal starte med UTF-8 BOM for Excel-kompatibilitet"
    );
    let header_line = csv_data
        .lines()
        .next()
        .expect("CSV skal have en overskriftslinje");
    assert!(
        header_line.contains("Foretrukken term")
            && header_line.contains("Accepteret term")
            && header_line.contains("Frarådet term")
            && header_line.contains("Definition")
            && header_line.contains("Juridisk kilde")
            && header_line.contains("Afledt af"),
        "CSV overskrift skal indeholde samtlige 12 FDA standardkolonner jf. Bilag D/E"
    );
    assert!(csv_data.contains("Køretøj"), "CSV skal indeholde 'Køretøj'");
    assert!(
        csv_data.contains("Transportmiddel"),
        "CSV skal indeholde 'Transportmiddel'"
    );
    // Test RFC-4180 escaping af citationstegn: "stelnummer" -> ""stelnummer""
    assert!(
        csv_data.contains("\"\"stelnummer\"\""),
        "RFC-4180 kræver dobbelte citationstegn ved escaping af anførselstegn i CSV felter"
    );

    // === AC3: Samlet Modelrapport (Markdown & HTML) ===
    let md_report = export_model_report_markdown(&project);
    assert!(
        md_report.contains("# Køretøjsmodellen"),
        "Markdown rapport skal have modelnavn som H1"
    );
    assert!(
        md_report.contains("## Indholdsfortegnelse"),
        "Markdown rapport skal have indholdsfortegnelse"
    );
    assert!(
        md_report.contains("Tabel D: Modelmetadata") || md_report.contains("Modelmetadata"),
        "Rapport skal have Tabel D"
    );
    assert!(
        md_report.contains("Motorstyrelsen"),
        "Rapport skal indeholde ansvarlig organisation"
    );
    assert!(
        md_report.contains("Færdselsloven § 2, stk. 1"),
        "Rapport skal indeholde lovgrundlag"
    );
    assert!(
        md_report.contains("Personbil"),
        "Rapport skal liste begreber og klasser"
    );

    let html_report = export_model_report_html(&project);
    assert!(
        html_report.contains("<!DOCTYPE html>"),
        "HTML rapport skal være gyldigt HTML5 dokument"
    );
    assert!(
        html_report.contains("<title>Køretøjsmodellen</title>"),
        "HTML rapport skal have korrekt titel"
    );
    assert!(
        html_report.contains("charset=\"utf-8\""),
        "HTML rapport skal specificere utf-8 tegnsæt"
    );
    assert!(
        html_report.contains("Motorstyrelsen"),
        "HTML rapport skal indeholde modelmetadata"
    );

    // === AC4 & AC5: UI Message Dispatch Flow ===
    let mut app = App::new_with_path(None);
    // MenuType::Export skal være tilgængelig
    assert_eq!(app.active_menu(), None);
    let _ = app.update(Message::ToggleMenu(kant::ui::app::MenuType::Export));
    assert_eq!(app.active_menu(), Some(kant::ui::app::MenuType::Export));
}

#[test]
fn test_task_060_machine_readable_model_interchange_xmi_skos_and_shacl() {
    // 1. Opret komplet testmodel
    let mut metadata = ModelMetadata::new(
        "Køretøjsmodellen",
        "Kernemodel for køretøjer og registrering i Danmark",
        "https://data.gov.dk/model/core/vehicle",
        "Motorstyrelsen",
        "Transport og Trafik",
        "1.0.0",
        ModelStatus::Completed,
    );
    metadata.set_approval_status(ApprovalStatus::Approved);
    metadata.set_legal_sources(vec![
        "https://www.retsinformation.dk/eli/lta/2016/976".to_string()
    ]);
    let mut project = ModelProject::new(metadata);

    // Begreber
    let mut c1 = Concept::new(
        "Køretøj",
        "Et mobilt teknisk anlæg, der anvendes til transport af personer eller gods.",
        BelongsToDomain::Yes,
    );
    c1.set_accepted_term(Some("Transportmiddel".to_string()));
    c1.set_deprecated_term(Some("Vogn".to_string()));
    c1.set_example(Some("Bil, cykel, bus.".to_string()));
    c1.set_application_note(Some("Anvendes som overordnet begreb.".to_string()));
    c1.set_legal_source(Some("Færdselsloven § 2, stk. 1".to_string()));
    c1.set_identifier(Some(
        "https://data.gov.dk/model/core/vehicle/Koeretoej".to_string(),
    ));

    let mut c2 = Concept::new(
        "Personbil",
        "Et køretøj indrettet til befordring af højst 9 personer.",
        BelongsToDomain::Yes,
    );
    c2.set_identifier(Some(
        "https://data.gov.dk/model/core/vehicle/Personbil".to_string(),
    ));
    c2.set_comment(Some("Underlagt periodisk syn.".to_string()));

    let _id1 = project.add_concept(c1.clone()).unwrap();
    let _id2 = project.add_concept(c2.clone()).unwrap();

    // Begrebsgraf: Generalisering c2 (Personbil) -> c1 (Køretøj)
    let n1_id = project.concept_graph_mut().add_node(&c1);
    let n2_id = project.concept_graph_mut().add_node(&c2);
    project
        .concept_graph_mut()
        .add_relation(n2_id, n1_id, RelationKind::Generalization);

    // Informationsmodel: Enumeration
    let mut drivkraft_enum = InformationEnumeration::new(
        "DrivkraftType",
        vec![
            "El".to_string(),
            "Benzin".to_string(),
            "Diesel".to_string(),
            "Brint".to_string(),
        ],
    );
    drivkraft_enum.set_definition(Some(
        "Drivkraftkilde for et motoriseret køretøj.".to_string(),
    ));
    let enum_id = project
        .information_model_mut()
        .add_enumeration(drivkraft_enum);

    // Informationsmodel: Klasser & Attributter
    let mut koeretoej_class = InformationClass::from_concept(&c1);
    koeretoej_class.set_abstract(true);
    let class_koeretoej_id = project.information_model_mut().add_class(koeretoej_class);

    let mut personbil_class = InformationClass::from_concept(&c2);
    personbil_class.add_attribute(Attribute::new(
        "stelnummer",
        PrimitiveType::CharacterString,
        Multiplicity::exactly_one(),
    ));
    personbil_class.add_attribute(Attribute::new(
        "siddepladser",
        PrimitiveType::Integer,
        Multiplicity::exactly_one(),
    ));
    personbil_class.add_attribute(Attribute::new(
        "egenvaegtKg",
        PrimitiveType::Decimal,
        Multiplicity::zero_or_one(),
    ));
    personbil_class.add_attribute(Attribute::new(
        "drivkraft",
        InformationDataType::Enumeration {
            enumeration_id: enum_id,
        },
        Multiplicity::zero_or_one(),
    ));
    let class_personbil_id = project.information_model_mut().add_class(personbil_class);

    // Informationsgraf noder og generaliseringskant
    let cn1_id = project
        .information_graph_mut()
        .add_node(class_koeretoej_id, 0);
    let cn2_id = project
        .information_graph_mut()
        .add_node(class_personbil_id, 4);
    project.information_graph_mut().add_relation(
        cn2_id,
        cn1_id,
        RelationKind::Generalization,
        None,
    );

    // === AC1: UML 2.5 / XMI 2.1 Eksport til Enterprise Architect ===
    let xmi_output = export_to_xmi_2_1(&project);
    assert!(
        xmi_output.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"),
        "XMI skal starte med standard XML 1.0 prolog"
    );
    assert!(
        xmi_output.contains("<xmi:XMI xmi:version=\"2.1\" xmlns:uml=\"http://schema.omg.org/spec/UML/2.1\" xmlns:xmi=\"http://schema.omg.org/spec/XMI/2.1\">"),
        "XMI skal have standard OMG UML 2.1 / XMI 2.1 namespaces for Enterprise Architect kompatibilitet"
    );
    assert!(
        xmi_output.contains("<uml:Model"),
        "XMI skal have et uml:Model rodelement"
    );
    assert!(
        xmi_output.contains("name=\"Køretøjsmodellen\""),
        "XMI modelnavn skal matche projektnavn"
    );
    assert!(
        xmi_output.contains("<packagedElement xmi:type=\"uml:Package\""),
        "XMI skal indeholde en pakke for modellen"
    );
    assert!(
        xmi_output.contains("<packagedElement xmi:type=\"uml:Class\"")
            && xmi_output.contains("name=\"Personbil\""),
        "XMI skal indeholde klassen Personbil"
    );
    assert!(
        xmi_output.contains("<ownedAttribute xmi:type=\"uml:Property\"")
            && xmi_output.contains("name=\"stelnummer\""),
        "XMI skal indeholde attributten stelnummer som uml:Property"
    );
    assert!(
        xmi_output.contains("<lowerValue xmi:type=\"uml:LiteralInteger\"")
            && xmi_output.contains("<upperValue xmi:type=\"uml:LiteralUnlimitedNatural\""),
        "XMI skal specificere UML multipliciteter via lowerValue og upperValue"
    );
    assert!(
        xmi_output.contains("<packagedElement xmi:type=\"uml:Enumeration\"")
            && xmi_output.contains("name=\"DrivkraftType\""),
        "XMI skal serialisere kontrollerede udfaldsrum som uml:Enumeration"
    );
    assert!(
        xmi_output.contains("<ownedLiteral xmi:type=\"uml:EnumerationLiteral\"")
            && xmi_output.contains("name=\"El\""),
        "XMI skal indeholde EnumerationLiterals for værdierne"
    );
    assert!(
        xmi_output.contains("<generalization xmi:type=\"uml:Generalization\""),
        "XMI skal indeholde uml:Generalization mellem subklasse og superklasse"
    );

    // === AC2: W3C SKOS RDF/Turtle Eksport af Begrebsliste ===
    let skos_output = export_to_skos_turtle(&project);
    assert!(
        skos_output.contains("@prefix skos: <http://www.w3.org/2004/02/skos/core#> ."),
        "SKOS Turtle skal definere skos: namespace"
    );
    assert!(
        skos_output.contains("@prefix dct: <http://purl.org/dc/terms/> ."),
        "SKOS Turtle skal definere dct: namespace"
    );
    assert!(
        skos_output.contains("a skos:ConceptScheme ;"),
        "SKOS Turtle skal have et skos:ConceptScheme"
    );
    assert!(
        skos_output.contains("dct:title \"Køretøjsmodellen\"@da ;"),
        "SKOS ConceptScheme skal indeholde titlen med dansk sprogtag"
    );
    assert!(
        skos_output.contains("a skos:Concept ;"),
        "SKOS Turtle skal deklarere begreber som skos:Concept"
    );
    assert!(
        skos_output.contains("skos:prefLabel \"Køretøj\"@da ;"),
        "SKOS Concept skal have skos:prefLabel"
    );
    assert!(
        skos_output.contains("skos:altLabel \"Transportmiddel\"@da ;"),
        "SKOS Concept skal have skos:altLabel for accepteret term"
    );
    assert!(
        skos_output.contains("skos:hiddenLabel \"Vogn\"@da ;"),
        "SKOS Concept skal have skos:hiddenLabel for frarådet term"
    );
    assert!(
        skos_output.contains("skos:definition"),
        "SKOS Concept skal indeholde skos:definition"
    );
    assert!(
        skos_output.contains("skos:example \"Bil, cykel, bus.\"@da ;"),
        "SKOS Concept skal indeholde skos:example"
    );
    assert!(
        skos_output.contains("skos:scopeNote \"Anvendes som overordnet begreb.\"@da ;"),
        "SKOS Concept skal indeholde skos:scopeNote for anvendelsesnote"
    );
    assert!(
        skos_output.contains("skos:broader") || skos_output.contains("skos:narrower"),
        "SKOS Concept skal indeholde hierarkiske relationer (broader / narrower)"
    );

    // === AC3: W3C SHACL / OWL RDF/Turtle Eksport af Informationsmodel ===
    let shacl_output = export_to_shacl_turtle(&project);
    assert!(
        shacl_output.contains("@prefix sh: <http://www.w3.org/ns/shacl#> ."),
        "SHACL Turtle skal definere sh: namespace"
    );
    assert!(
        shacl_output.contains("@prefix owl: <http://www.w3.org/2002/07/owl#> ."),
        "SHACL Turtle skal definere owl: namespace"
    );
    assert!(
        shacl_output.contains("a owl:Ontology ;"),
        "SHACL Turtle skal deklarere ontologien som owl:Ontology"
    );
    assert!(
        shacl_output.contains("sh:NodeShape") && shacl_output.contains("owl:Class"),
        "Hver klasse skal modelleres som både owl:Class og sh:NodeShape"
    );
    assert!(
        shacl_output.contains("sh:targetClass"),
        "SHACL NodeShape skal have sh:targetClass"
    );
    assert!(
        shacl_output.contains("sh:property ["),
        "SHACL NodeShape skal have sh:property shapes"
    );
    assert!(
        shacl_output.contains("sh:path") && shacl_output.contains("stelnummer"),
        "SHACL property shape skal have sh:path for stelnummer"
    );
    assert!(
        shacl_output.contains("sh:datatype xsd:string"),
        "SHACL property for CharacterString skal referere til xsd:string"
    );
    assert!(
        shacl_output.contains("sh:datatype xsd:integer"),
        "SHACL property for Integer skal referere til xsd:integer"
    );
    assert!(
        shacl_output.contains("sh:minCount 1"),
        "SHACL property med 1..1 skal have sh:minCount 1"
    );
    assert!(
        shacl_output.contains("sh:maxCount 1"),
        "SHACL property med 1..1 eller 0..1 skal have sh:maxCount 1"
    );
    assert!(
        shacl_output.contains("sh:in ("),
        "SHACL property for Enumeration skal specificere tilladte værdier med sh:in"
    );

    // === AC4: UI Message Dispatch & Export Menu Integration ===
    let mut app = App::new_with_path(None);
    // Verificer at Export-menuen har nye maskinlæsbare valgmuligheder
    let _ = app.update(Message::ToggleMenu(kant::ui::app::MenuType::Export));
    assert_eq!(app.active_menu(), Some(kant::ui::app::MenuType::Export));

    // Verificer at dialog messages findes og kan håndteres
    let _ = app.update(Message::ExportModelXmiDialog);
    let _ = app.update(Message::ExportModelSkosDialog);
    let _ = app.update(Message::ExportModelShaclDialog);
}

#[test]
fn test_task_088_robust_excel_import() {
    use kant::features::concepts::excel::parse_concepts_from_xlsx_bytes;
    use rust_xlsxwriter::Workbook;

    // === Del 1: FDA-skabelon kompatibilitet (Begrebsliste DA vs Begrebsliste DA+EN) ===
    // En bruger downloader den officielle FDA-skabelon, udfylder kun 'Begrebsliste DA',
    // og lader 'Begrebsliste DA+EN' være tom for databegreber.
    let mut workbook = Workbook::new();

    let meta_sheet = workbook.add_worksheet();
    meta_sheet.set_name("Forretningsmetadata").unwrap();
    meta_sheet.write_string(0, 0, "Modelnavn").unwrap();
    meta_sheet.write_string(0, 1, "Transportmodel").unwrap();

    let da_sheet = workbook.add_worksheet();
    da_sheet.set_name("Begrebsliste DA").unwrap();
    // Række 0: SKOS overskrifter
    da_sheet.write_string(0, 0, "prefLabel (da)").unwrap();
    da_sheet.write_string(0, 1, "definition (da)").unwrap();
    // Række 1: Danske overskrifter
    da_sheet.write_string(1, 0, "Foretrukken term").unwrap();
    da_sheet.write_string(1, 1, "Definition").unwrap();
    // Række 2: Tom / mellemrum som i FDA skabelonen
    da_sheet.write_string(2, 0, " ").unwrap();
    // Række 3: Brugerens data 1 (med ordet 'definition' og 'forklaring' i teksten)
    da_sheet.write_string(3, 0, "Køretøj").unwrap();
    da_sheet
        .write_string(3, 1, "En definition på et transportmiddel på hjul.")
        .unwrap();
    // Række 4: Brugerens data 2 (term der starter med 'Foretrukken', definition starter med 'Definition')
    da_sheet
        .write_string(4, 0, "Foretrukken leverandør")
        .unwrap();
    da_sheet
        .write_string(
            4,
            1,
            "Definition: En leverandør der prioriteres i organisationens indkøb.",
        )
        .unwrap();

    // Fanen Begrebsliste DA+EN tilføjes også (som i skabelonen), men er tom for databegreber
    let da_en_sheet = workbook.add_worksheet();
    da_en_sheet.set_name("Begrebsliste DA+EN").unwrap();
    da_en_sheet.write_string(0, 0, "prefLabel (da)").unwrap();
    da_en_sheet.write_string(0, 3, "definition (da)").unwrap();
    da_en_sheet.write_string(0, 9, "prefLabel (en)").unwrap();
    da_en_sheet.write_string(0, 12, "definition (en)").unwrap();
    da_en_sheet
        .write_string(1, 0, "Foretrukken term (da)")
        .unwrap();
    da_en_sheet.write_string(1, 3, "Definition (da)").unwrap();

    let bytes = workbook.save_to_buffer().unwrap();

    // Test import
    let summary = parse_concepts_from_xlsx_bytes(&bytes)
        .expect("Skal kunne parse Excel dokumentet uden fejl");

    assert_eq!(
        summary.sheet_name, "Begrebsliste DA",
        "Parseren skal automatisk vælge det ark der reelt indeholder data ('Begrebsliste DA') frem for et tomt ark"
    );
    assert_eq!(
        summary.valid_concepts.len(),
        2,
        "Begge begreber fra Begrebsliste DA skal importeres som gyldige begreber"
    );
    assert_eq!(
        summary.rejected_rows.len(),
        0,
        "Ingen gyldige begreber må kastes bort som headers eller manglende felter"
    );

    let c1 = &summary.valid_concepts[0];
    assert_eq!(c1.preferred_term(), "Køretøj");
    assert_eq!(
        c1.definition(),
        "En definition på et transportmiddel på hjul."
    );

    let c2 = &summary.valid_concepts[1];
    assert_eq!(c2.preferred_term(), "Foretrukken leverandør");
    assert_eq!(
        c2.definition(),
        "Definition: En leverandør der prioriteres i organisationens indkøb."
    );
}

