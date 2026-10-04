---
type: Task Package
title: "Task 072: Komplet Tastaturnavigation for Begrebsliste, Inspektorer og Modaler"
description: "Fuldendt tastaturstyring for tabelrækker i Begrebsliste, alle elementer i Informationsmodellens Egenskabspanel samt modaler for historik og samarbejde."
status: done
generated: { by: process:antigravity-task-init, at: "2026-10-04T21:37:00Z" }
tags: [ui, keyboard, focus, accessibility, a11y, table, inspector, modals]
---

# Task 072: Komplet Tastaturnavigation for Begrebsliste, Inspektorer og Modaler

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-10-04`

## 🎯 Formål
Sikre 100% tastaturnavigation til samtlige interaktive kontroller i applikationen:
1. **Regel for Luk-kryds ("✕") i modaler og paneler**:
   - Titellinjernes luk-kryds nås **ikke** med Tab, men lukkes konsekvent og ergonomisk via <kbd>Esc</kbd>.
2. **Start Live Session (Vært) (`start_session_modal`)**:
   - "Relay server preset" dropdown nås med <kbd>Tab</kbd>, omkranses af den 1.5px orange fokusring (`ThemeColors::FOCUS_RING`), og <kbd>Enter</kbd> / <kbd>Mellemrum</kbd> cykler server presets.
   - Checkboxen for *"Husk servervalg til fremtidige sessioner"* gøres fokuserbar med orange fokusring og kan toggles med <kbd>Enter</kbd> / <kbd>Mellemrum</kbd>.
   - Custom WebSocket-URL tekstfeltet indgår når aktivt.
   - Handlingsknapperne: Kopiér sessionskode, Annuller og Start Session.
3. **1. Begrebsliste (Tabelrækker)**:
   - <kbd>Tab</kbd> fra værktøjslinjen fortsætter ned i tabellen over de filtrerede begreber.
   - For hver række: `✏️ Rediger` og `🗑️ Slet` modtager 1.5px orange fokusring.
   - <kbd>Enter</kbd> eller <kbd>Mellemrum</kbd> på `Rediger` åbner begrebseditoren for det valgte begreb.
   - <kbd>Enter</kbd> eller <kbd>Mellemrum</kbd> på `Slet` sletter begrebet.
4. **3. Informationsmodel > Egenskaber (Klasse & Enumeration)**:
   - **Klasse**:
     - Klassenavn & Beskrivelse (`info_class_name_input`, `info_class_desc_input`).
     - Abstrakt klasse checkmark med orange fokusring og toggle via <kbd>Enter</kbd> / <kbd>Mellemrum</kbd>.
     - Klassetype (`Lokal` / `Indlånt`) samt eventuelt kildemodelfelt.
     - Lærredshandlinger (`Tilføj/Fjern fra diagram`, `🗑️ Slet klasse`).
     - Tilknyttede begreber: Fjern-knap (`✕`) for hvert begreb.
     - `+ Knyt begreb` picker omkranses af 1.5px orange fokusring og knytter begreb via <kbd>Enter</kbd> / <kbd>Mellemrum</kbd>.
     - Attributter: `+ Tilføj attribut` knap samt hvert attributs tekstfelt og slet-knap.
   - **Enumeration**:
     - Enumerationsnavn & Valgfri definition (`info_enum_name_input`, `info_enum_def_input`).
     - Lærredshandlinger (`Tilføj/Fjern fra diagram`, `🗑️ Slet enumeration`).
     - Ny værdi tekstfelt (`new_enum_val_input`) og `+ Tilføj` knap.
     - Slet-knapper (`✕`) for samtlige oprettede værdier i listen.

## 📋 Acceptance Criteria
- [x] **AC1 - Begrebsliste Tabelrækker tastaturnavigation**:
  - Tab cykler fra værktøjslinjens knapper videre til begrebsrækkernes Rediger- og Slet-knapper.
  - Hver fokuseret knap viser den 1.5px orange fokusring.
  - Enter eller Mellemrum på Rediger åbner ConceptEditor med det valgte begreb.
  - Enter eller Mellemrum på Slet fjerner begrebet.
- [x] **AC2 - Start Live Session kontroller & Preset Dropdown**:
  - Preset dropdown kan fokuseres med Tab, viser orange fokusring, og Enter cykler presets.
  - Checkboxen for "Husk servervalg" kan fokuseres med Tab, viser orange fokusring og kan toggles med Enter eller Mellemrum.
  - Luk-kryds i titellinje ekskluderes fra Tab og vinduet lukkes med Esc.
- [x] **AC3 - Modelhistorik tidslinje navigation**:
  - Eventuel "Commit ændringer"-knap og de enkelte commit-kort i listen kan fokuseres med Tab og viser orange fokusring.
  - Luk-kryds ekskluderes fra Tab og vinduet lukkes med Esc eller Luk-knap.
- [x] **AC4 - Informationsmodel Egenskabspanel (Klasse & Enumeration)**:
  - Klasse: Abstrakt checkbox, Lokal/Indlånt, begrebs slet-knapper, `+ Knyt begreb` picker samt attribut-knapper kan fokuseres og aktiveres med tastatur.
  - Enumeration: Navn, definition, canvas-handling, slet enumeration, ny værdi input, tilføj værdi samt slet-knapper for alle værdier kan gennemcykles og betjenes med Tab/Enter.
- [x] **AC5 - Regression Gauntlet**:
  - Samtlige nye flows verificeres med 5 automatiserede accepttests i `tests/acceptance.rs`.
  - `cargo test --workspace` og `cargo clippy --workspace -- -D warnings` forbliver 100% grønne uden warnings.

## 🚫 Must NOT
- Må IKKE bryde almindelig tekstindtastning i aktive tekstfelter.
- Må IKKE trigge knap- eller checkbox-aktivering når fokus er i et tekstfelt (mellemrum skal skrive mellemrumstegn i tekstfelter).
- Må IKKE inkludere titellinjers luk-kryds ("✕") i Tab-cyklussen (lukning sker via Esc).
- Må IKKE lave destruktive git handlinger eller remote push.

## 📝 Revisions
- 2026-10-04: Oprettet opgavepakke efter brugerfeedback for at dække samtlige interaktive kontroller i tabeller, inspektorer og modaler.
- 2026-10-04: Forfinet navigation efter brugerinput: Luk-krydser ("✕") fjernet fra Tab-cyklus til fordel for Esc, "Relay server preset" dropdown tilføjet til cyklus med Enter-cykling, "+ Knyt begreb" picker tilføjet for klasseinspektor, og komplet tastaturfokuscyklus implementeret for Enumeration.
- 2026-10-04: Verificeret med 5 accepttests i `tests/acceptance.rs`, samt 100% grøn workspace test suite og zero clippy warnings.

## 🧪 Verifikation
- `cargo test --test acceptance test_task_072_`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
