---
type: Task Package
title: "Task 087: Concept Studio and Model Decomposition"
description: "Dekomponering af concept_editor.rs (879 l), excel.rs (833 l), concept_model_view.rs (889 l) og edge_router.rs (866 l) i Package-by-Feature slices under 800 linjer samt fjernelse af 4 undtagelser i gauntlet.toml"
status: done
generated: { by: process:antigravity-task-init, at: "2026-10-18T22:00:00Z" }
tags: [refactor, concepts, concept-model, decomposition, modularity, architecture, ratchet]
scope:
  - "src/ui/concept_editor.rs"
  - "src/ui/concept_model_view.rs"
  - "src/ui/edge_router.rs"
  - "src/features/concepts/**"
  - "src/features/concept_model/**"
  - "src/ui/canvas/**"
  - "gauntlet.toml"
  - "tests/architecture_test.rs"
test_refs:
  - "tests/features/concept_decomposition_acceptance_test.rs"
  - "tests/architecture_test.rs"
---

# Task 087: Concept Studio and Model Decomposition

**Status**: `DONE`  
**Intent**: `🔄 REFACTOR`  
**Oprettet**: `2026-10-18`  

## 🎯 Formål
Dekomponere de 4 store monolitter inden for Begrebslisten (Fase 1) og Begrebsmodellen (Fase 2) jf. `Package-by-Feature` princippet og Kants opdaterede domæneglossary i `CONTEXT.md`:
1. Opdele `src/ui/concept_editor.rs` (879 linjer) i modulære komponenter under `src/features/concepts/ui/` ($\le 500$ linjer).
2. Opdele `src/features/concepts/excel.rs` (833 linjer) i modulære enheder under `src/features/concepts/excel/` for import, eksport og skabelongenerering ($\le 500$ linjer).
3. Opdele `src/ui/concept_model_view.rs` (889 linjer) i palet, inspektør og orkestrator under `src/features/concept_model/ui/` ($\le 500$ linjer).
4. Dekomponere `src/ui/edge_router.rs` (866 linjer) til modulære rute- og kanalberegnere under `src/features/concept_model/routing/` eller `src/ui/canvas/` ($\le 500$ linjer).
5. Fjerne alle 4 undtagelser fra `gauntlet.toml`, så kun 4 undtagelser resterer i hele systemet.

## 📋 Acceptance Criteria
- [x] **AC1 (Begrebsliste Editor Dekomponering)**: Dekomponer `src/ui/concept_editor.rs` i modulære UI-komponenter under `src/features/concepts/ui/` (`modal.rs`, `form.rs` osv. $\le 500$ linjer) med tynd facade $\le 400$ linjer.
- [x] **AC2 (Excel Import & Eksport Dekomponering)**: Dekomponer `src/features/concepts/excel.rs` i `importer.rs`, `exporter.rs`, `template.rs` under `src/features/concepts/excel/` ($\le 500$ linjer hver) med ren facade i `mod.rs`.
- [x] **AC3 (Begrebsmodel Studio Dekomponering)**: Dekomponer `src/ui/concept_model_view.rs` i palet, inspektør og view under `src/features/concept_model/ui/` ($\le 500$ linjer hver) med tynd facade $\le 400$ linjer.
- [x] **AC4 (Edge Router Dekomponering)**: Dekomponer `src/ui/edge_router.rs` i modulære routing-enheder (`routing.rs`, `bus.rs`, `geometry.rs` $\le 500$ linjer hver) med facade $\le 400$ linjer.
- [x] **AC5 (Fjernelse af 4 Undtagelser i gauntlet.toml)**: Fjern undtagelserne for `concept_editor.rs`, `excel.rs`, `concept_model_view.rs` og `edge_router.rs` fra `gauntlet.toml` og opdater `tests/architecture_test.rs`.
- [x] **AC6 (Regression & Verifikation)**: Samtlige enheds- og accepttests i workspace passerer uden fejl, og `xgauntlet check-architecture` rapporterer 0 overtrædelser.

## 🚫 Must NOT
- Må IKKE ændre FDA begrebsmodellens valideringsregler, Aristoteles-linter eller Excel-import-/eksportformater (frosne assertions).
- Må IKKE ændre eksisterende Manhattan ortogonal ruteberegning eller hysteresis-adfærd.
- Må IKKE skabe nye filer over 800 linjer eller testfiler over 1.000 linjer.
- Må IKKE lempes på nogen spærrehager.

## 📝 Revisions
- 2026-10-18: Oprettet opgavepakke Task 087 til dekomponering af Begrebslisten og Begrebsmodellens monolitter.
- 2026-10-18: Implementeret dekomponering i slices under 500 linjer, fjernet 4 undtagelser i gauntlet.toml, bestået alle accepttests og verificeret arkitektur.

## 🧪 Verifikation
- `cargo test --workspace`
- `cargo test --test architecture_test`
- `xgauntlet check-architecture`
- `xgauntlet verify --task 087-concept-studio-and-model-decomposition`
- `xgauntlet check-spec -t 087-concept-studio-and-model-decomposition`
