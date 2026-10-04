---
type: Task Package
title: "Task 069: Ensartet Tastaturhierarki (Hierarkisk Esc-afvikling, Enter-hurtigredigering på canvas og komplet modal-lukning)"
description: "Implementering af ensartet tastaturnavigation med hierarkisk Esc-afvikling, Enter-hurtigredigering på canvas for noder og relationer, komplet Esc-lukning af alle 9 modaler samt fokuslanding på faneblade"
status: active
generated: { by: process:antigravity-task-init, at: "2026-10-04T11:48:00Z" }
tags: [ui, keyboard, shortcuts, accessibility, ergonomics, canvas, modals, navigation]
---

# Task 069: Ensartet Tastaturhierarki (Hierarkisk Esc-afvikling, Enter-hurtigredigering på canvas og komplet modal-lukning)

**Status**: `ACTIVE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-10-04`

## 🎯 Formål
At etablere et professionelt, ensartet og forudsigeligt tastaturhierarki på tværs af hele Kant applikationen:
1. Hurtig aktivering af redigering via `Enter` på canvas for både noder (`preferred_term_input` / `info_class_name_input`) og relationer (`edge_label_input` / `info_edge_label_input`).
2. Robust, deterministisk `Escape`-afvikling lag for lag: aktivt input -> inline editor -> modale dialoger (alle 9 modaler inkl. diagnostik) -> canvas afmarkering -> landing på de 3 primære faneblade.
3. Direkte navigation til de 3 faneblade (`Ctrl+1`, `Ctrl+2`, `Ctrl+3`) samt fokus-id'er til tab-baren.

## 📋 Acceptance Criteria
- [ ] **AC1 - Enter-hurtigredigering på canvas**:
  - Når en node er markeret på canvas (`selected_graph_node_id`), åbner `Enter` den inline koncepteditor (`is_inline_graph_editing = true`, `editor_state = Some(...)`) og fokuserer `preferred_term_input`.
  - Når en relation/edge er markeret på canvas (`selected_edge`), fokuserer `Enter` feltet `edge_label_input` i højre inspektørpanel.
  - I informationsmodellen: Valgt informationsklasse fokuserer `info_class_name_input` og valgt relation fokuserer `info_edge_label_input`.
- [ ] **AC2 - Komplet Esc-lukning af samtlige 9 modaler**:
  - Tastatur `Escape` lukker altid en åben modal uden sideeffekter.
  - Omfatter alle 9 modaler:
    1. Diagnostics modal (`diagnostics_modal`)
    2. Publish modal (`publish_modal`)
    3. Model / element history modal (`history_modal`)
    4. Conflict resolver modal (`conflict_resolver_modal`)
    5. Git connection modal (`git_connection_modal`)
    6. Git clone modal (`git_clone_modal`)
    7. Metadata modal (`metadata_modal`)
    8. Start session modal (`start_session_modal`)
    9. Join session modal (`join_session_modal`)
    samt øvrige overlay-dialoger (Excel import preview, Excel schema guide, relation dialogs, quick create, file dialog, burger-/topmenuer).
- [ ] **AC3 - Hierarkisk Esc-afvikling lag for lag**:
  - Lag 1 (Aktivt input): Når et inputfelt har fokus, afbryder Esc fokustilstanden (defokus).
  - Lag 2 (Inline editor): Når en inline graf-editor er åben uden inputfokus, lukker Esc editoren (`is_inline_graph_editing = false`, `editor_state = None`), mens noden bevarer sin markering.
  - Lag 3 (Modaler / menuer): Når en modal eller menu er åben, lukker Esc denne modal/menu.
  - Lag 4 (Canvas afmarkering): Når elementer (noder eller kanter) er markeret på canvas, fjerner Esc alle markeringer.
  - Lag 5 (Landing på de 3 primære faneblade): Når canvas er ryddet og ingen dialoger/editorer er åbne, flytter Esc fokus til det aktive faneblad på topbaren (`tab_concept_list`, `tab_concept_model`, `tab_information_model`).
- [ ] **AC4 - Ergonomisk tastaturnavigation til faneblade**:
  - Fanebladsknapperne har veldefinerede id'er (`tab_concept_list`, `tab_concept_model`, `tab_information_model`).
  - Globale genveje `Ctrl+1`, `Ctrl+2`, `Ctrl+3` skifter direkte til hhv. Begrebsliste, Begrebsdiagram og Informationsmodel.
- [ ] **AC5 - Verifikation & Regression Gauntlet**:
  - Samtlige accepttests i `cargo test --test acceptance test_task_069_` samt hele testsuiten passerer grønt.

## 🚫 Must NOT
- Må IKKE fjerne eller ændre eksisterende globale genveje som `Ctrl+S` (gem), `Ctrl+B` (palet toggle) eller zoom `Ctrl++` / `Ctrl+-` / `Ctrl+0`.
- Må IKKE lukke en modal hvis brugeren kun ønskede at forlade et aktivt inputfelt (lag-for-lag afvikling sikres).
- Må IKKE introducere blocking I/O eller headless CI regressionsfejl.

## 📝 Revisions
- 2026-10-04: Oprettet opgavepakke efter brugergodkendelse.

## 🧪 Verifikation
- `cargo test --test acceptance test_task_069_`
- `cargo check --workspace`
- `cargo test --workspace`
