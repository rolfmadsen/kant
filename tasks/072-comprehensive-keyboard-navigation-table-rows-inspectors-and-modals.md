---
type: Task Package
title: "Task 072: Komplet Tastaturnavigation for Begrebsliste, Inspektorer og Modaler"
description: "Fuldendt tastaturstyring for tabelrækker i Begrebsliste, alle elementer i Informationsmodellens Egenskabspanel samt modaler for historik og samarbejde."
status: active
generated: { by: process:antigravity-task-init, at: "2026-10-04T21:37:00Z" }
tags: [ui, keyboard, focus, accessibility, a11y, table, inspector, modals]
---

# Task 072: Komplet Tastaturnavigation for Begrebsliste, Inspektorer og Modaler

**Status**: `ACTIVE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-10-04`

## 🎯 Formål
Sikre 100% tastaturnavigation til samtlige interaktive kontroller i applikationen:
1. **Modelhistorik & Tidslinje (`view_history_modal`)**:
   - Titellinjens luk-kryds (`✕`) gøres fokuserbar.
   - Ved tom historik: Knappen `"🚀 Commit og send ændringer nu..."` indgår i cyklussen.
   - Ved historik med commits: De enkelte commit-kort gøres fokuserbare med den 1.5px orange fokusring (`ThemeColors::FOCUS_RING`).
   - "Luk"-knappen i bunden bevarer sit fokus.
2. **Start Live Session (Vært) (`start_session_modal`)**:
   - Titellinjens luk-kryds (`✕`) gøres fokuserbar.
   - Checkboxen for *"Husk servervalg til fremtidige sessioner"* gøres fokuserbar med orange fokusring og kan toggles med <kbd>Enter</kbd> / <kbd>Mellemrum</kbd>.
   - Custom WebSocket-URL tekstfeltet indgår når aktivt.
   - Handlingsknapperne: Kopiér sessionskode, Annuller og Start Session.
3. **1. Begrebsliste (Tabelrækker)**:
   - <kbd>Tab</kbd> fra værktøjslinjen fortsætter ned i tabellen over de filtrerede begreber.
   - For hver række: `✏️ Rediger` og `🗑️ Slet` modtager 1.5px orange fokusring.
   - <kbd>Enter</kbd> eller <kbd>Mellemrum</kbd> på `Rediger` åbner begrebseditoren for det valgte begreb.
   - <kbd>Enter</kbd> eller <kbd>Mellemrum</kbd> på `Slet` sletter begrebet.
4. **3. Informationsmodel > Egenskaber (Komplet Inspektorpanelet)**:
   - **Klassenavn & Beskrivelse**: Tekstfelter (`info_class_name_input`, `info_class_desc_input`).
   - **Abstrakt klasse checkmark**: Modtager orange fokusring og toggles med <kbd>Enter</kbd> / <kbd>Mellemrum</kbd> (`Message::SetInformationClassAbstract`).
   - **Klassetype**: Knapperne `Lokal (Sand)` og `Indlånt (Blå)` modtager orange ring og aktiveres med <kbd>Enter</kbd> / <kbd>Mellemrum</kbd>. Hvis indlånt: tekstfelt for kildemodel (`info_class_origin_model_input`).
   - **Lærredshandlinger**: `Tilføj/Fjern fra diagram` og `🗑️ Slet klasse` knapper.
   - **Tilknyttede begreber**: Hvert tilknyttet begrebs fjern-knap (`✕`) modtager orange fokusring og fjerner koblingen på Enter.
   - **Attributter**:
     - `+ Tilføj attribut` knap.
     - For hver attribut: Attributnavn-felt samt slet-knap (`✕`) modtager fokus med orange ring.

## 📋 Acceptance Criteria
- [ ] **AC1 - Begrebsliste Tabelrækker tastaturnavigation**:
  - Tab cykler fra værktøjslinjens knapper videre til begrebsrækkernes Rediger- og Slet-knapper.
  - Hver fokuseret knap viser den 1.5px orange fokusring.
  - Enter eller Mellemrum på Rediger åbner ConceptEditor med det valgte begreb.
  - Enter eller Mellemrum på Slet fjerner begrebet.
- [ ] **AC2 - Start Live Session kontroller**:
  - Titellinjens luk-kryds ("✕") og checkboxen for "Husk servervalg" kan fokuseres med Tab.
  - Checkboxen viser den orange fokusring og kan toggles med Enter eller Mellemrum.
- [ ] **AC3 - Modelhistorik tidslinje navigation**:
  - Titellinjens luk-kryds ("✕"), eventuel "Commit ændringer"-knap og de enkelte commit-kort i listen kan fokuseres med Tab og viser orange fokusring.
- [ ] **AC4 - Informationsmodel Egenskabspanel fuld dækning**:
  - Abstrakt klasse checkbox kan fokuseres og toggles med Enter/Mellemrum.
  - Klassetype-knapper (Lokal/Indlånt) kan fokuseres og aktiveres med Enter/Mellemrum.
  - Tilknyttede begrebers fjern-knap ("✕") kan fokuseres og aktiveres med Enter/Mellemrum.
  - "+ Tilføj" attribut knap samt hver enkelt attributs tekstfelt og slet-knap kan fokuseres og aktiveres.
- [ ] **AC5 - Regression Gauntlet**:
  - Samtlige nye flows verificeres med automatiserede accepttests i `tests/acceptance.rs`.
  - `cargo test --workspace` og `cargo clippy --workspace -- -D warnings` forbliver 100% grønne uden warnings.

## 🚫 Must NOT
- Må IKKE bryde almindelig tekstindtastning i aktive tekstfelter.
- Må IKKE trigge knap- eller checkbox-aktivering når fokus er i et tekstfelt (mellemrum skal skrive mellemrumstegn i tekstfelter).
- Må IKKE lave destruktive git handlinger eller remote push.

## 📝 Revisions
- 2026-10-04: Oprettet opgavepakke efter brugerfeedback for at dække samtlige interaktive kontroller i tabeller, inspektorer og modaler.

## 🧪 Verifikation
- `cargo test --test acceptance test_task_072_`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
