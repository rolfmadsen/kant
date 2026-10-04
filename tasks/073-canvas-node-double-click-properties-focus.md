---
type: Task Package
title: "Task 073: Dobbeltklik på Node Åbner Egenskaber med Fokus i Navn"
description: "Dobbeltklik på en node i enten Begrebsmodel eller Informationsmodel (Klasse eller Enumeration) åbner egenskaber/inspektor med direkte tastaturfokus i navnefeltet, svarende til Enter-adfærden."
status: active
generated: { by: process:antigravity-task-init, at: "2026-10-04T22:33:00Z" }
tags: [ui, canvas, double-click, mouse, focus, inspector, information-model, concept-model]
---

# Task 073: Dobbeltklik på Node Åbner Egenskaber med Fokus i Navn

**Status**: `ACTIVE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-10-04`

## 🎯 Formål
Gøre museinteraktionen symmetrisk med tastaturadfærden (<kbd>Enter</kbd>):
1. **Begrebsmodel (`2. Begrebsmodel`)**:
   - Dobbeltklik på en begrebsnode åbner egenskaber/hurtigredigering i inspektoren og sætter omgående tastaturfokus i feltet "Foretrukken term" (`preferred_term_input`).
2. **Informationsmodel (`3. Informationsmodel`)**:
   - Dobbeltklik på en klassenode åbner klassens egenskabspanel og sætter tastaturfokus i feltet "Klassenavn" (`info_class_name_input`).
   - Dobbeltklik på en enumerationsnode åbner enumerationens egenskabspanel og sætter tastaturfokus i feltet "Enumerationsnavn" (`info_enum_name_input`).
3. **DiagramCanvas Ergonomi**:
   - Forbedre tolerance og naturlig dobbeltklik-takt (500 ms standard OS-interval og 20 px bevægelsestolerance).
   - Rydde påbegyndt `dragging_node` og `bulk_drag` ved dobbeltklik så museslip ikke forstyrrer fokus eller flytter noder utilsigtet.

## 📋 Acceptance Criteria
- [ ] **AC1 - Informationsmodel Dobbeltklik på Klasse**: Dobbeltklik på en klasse på lærredet vælger noden og sætter fokus i `info_class_name_input`.
- [ ] **AC2 - Informationsmodel Dobbeltklik på Enumeration**: Dobbeltklik på en enumeration på lærredet vælger noden og sætter fokus i `info_enum_name_input`.
- [ ] **AC3 - Begrebsmodel Dobbeltklik på Begreb**: Dobbeltklik på en begrebsnode åbner inline editor og sætter fokus i `preferred_term_input`.
- [ ] **AC4 - Multi-selection og relation nulstilling**: Dobbeltklik isolerer markeringen til den klikkede node og fjerner eventuel valgt relation.
- [ ] **AC5 - Regression Gauntlet**: Alle eksisterende og nye tests i `--workspace` passerer 100% grønt med zero clippy warnings.

## 🚫 Must NOT
- Må IKKE ødelægge almindelig enkeltklik-markering eller nodetræk (drag & drop).
- Må IKKE fejle hvis noden allerede er valgt.
- Må IKKE udføre remote push eller destruktive git handlinger.

## 📝 Revisions
- 2026-10-04: Oprettet opgavepakke efter brugerønske om dobbeltklik på noder svarende til Enter-adfærden.

## 🧪 Verifikation
- `cargo test test_task_073`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
