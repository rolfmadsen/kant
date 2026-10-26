---
type: Task Package
title: "Task 089: Excel Import Merge Awareness and New Model Option"
description: "Styrkelse af brugerforståelse og usability ved Excel-import: synliggørelse af fletning med den åbne model via målmodel-banner, uændrede begreber tæller, 'Flet med åben model' handlingsknap og 'Opret som ny model i stedet' alternativ."
status: done
generated: { by: process:antigravity-task-init, at: "2026-10-26T18:12:00Z" }
tags: [usability, excel, import, merge, model, workflow]
scope:
  - "src/features/export/ui/**"
  - "src/ui/app.rs"
  - "tests/features/export_acceptance_test.rs"
test_refs:
  - "tests/features/export_acceptance_test.rs"
---

# Task 089: Excel Import Merge Awareness and New Model Option

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-10-26`

## 🎯 Formål
Gøre det 100% gennemskueligt for brugeren, at en Excel-import som udgangspunkt fletter ind i den aktuelt åbne model, indikere præcis hvilken model der modtager begreberne, synliggøre hvor mange begreber der forbliver uændrede, samt give et direkte valg til i stedet at oprette en helt ny model ud fra Excel-arket.

## 📋 Acceptance Criteria
- [x] **AC1 - Målmodel-banner (Target Model Banner)**: Forhåndsvisningsdialogen viser tydeligt navnet på den model, der flettes ind i (f.eks. `🎯 Fletningsmål: "<Modelnavn>"` eller `"Navnløs model"`), så brugeren aldrig er i tvivl om hvilken model der beriges.
- [x] **AC2 - Tæller for Uændrede Begreber**: Statusrækken udvides med en fjerde indikator/pille: `🛡️ Uændrede i modellen: X`, som viser antallet af eksisterende begreber i den åbne model, som forbliver uberørt af importen.
- [x] **AC3 - Eksplicit Flet-Handling**: Bekræft-knappen er tydeligt mærket `"Flet med åben model"`, så handlingen mentalt kobles til en merge-operation.
- [x] **AC4 - Opret som Ny Model Alternativ**: Dialogen tilbyder en dedikeret sekundær handling: `"Opret som ny model i stedet"` (`Message::ConfirmExcelImportAsNewModel`). Ved aktivering nulstilles og initialiseres en ny model, eventuelle metadata fra Excel (eller filnavn) tilknyttes, alle gyldige begreber importeres rent, og dialogen lukkes.
- [x] **AC5 - Tastaturnavigation & Fokuscyklus**: Modalens fokuscyklus understøtter 3 handlinger: `0: Annuller`, `1: Opret som ny model i stedet`, `2: Flet med åben model` (standardfokus). Tryk på Enter bekræfter den fokuserede handling.

## 🚫 Must NOT
- Må IKKE ændre eksisterende begrebers interne `Uuid` under fletning.
- Må IKKE overse eller tabe data fra Excel-dokumentet ved hverken fletning eller oprettelse af ny model.
- Må IKKE overtræde arkitektur- og filstørrelsesgrænser jf. xGauntlet invariants.

## 📝 Revisions
- 2026-10-26: Oprettet opgavepakke efter usability-sparring om merge-awareness ved Excel-import.
- 2026-10-26: Tilføjet fletningsmål-banner, pille for uændrede begreber i åben model, 'Flet med åben model' handling samt 'Opret som ny model i stedet' alternativ med komplet testdækning.

## 🧪 Verifikation
- `cargo test --test export_acceptance test_task_089_excel_import_merge_awareness`
- `cargo test --test export_acceptance`
- `cargo clippy --all-targets`
- `cargo fmt --check`
- `xgauntlet check-spec`
- `xgauntlet verify --task 089`
