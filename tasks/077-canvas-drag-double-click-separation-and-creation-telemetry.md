---
type: Task Package
title: "Task 077: Adskillelse af Canvas Drag og Dobbeltklik samt Node-oprettelse Telemetri"
description: "Robustgørelse af DiagramCanvas museinteraktion så hurtigt træk efter valg ikke mistolkes som dobbeltklik, samt telemetri-logning ved lynoprettelse af begreber."
status: active
generated: { by: process:antigravity-task-init, at: "2026-10-06T19:35:00Z" }
tags: [canvas, drag-and-drop, double-click, mouse, telemetry, concept-model, information-model]
---

# Task 077: Adskillelse af Canvas Drag og Dobbeltklik samt Node-oprettelse Telemetri

**Status**: `ACTIVE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-10-06`
**Scope**: `src/ui/diagram_canvas.rs`, `src/ui/app.rs`, `tests/acceptance.rs`

## 🎯 Formål
Løse race condition og ergonomisk fejl i `DiagramCanvas`, hvor hurtigt træk efter valg mistolkes som dobbeltklik og afbryder nodetræk:
1. **Adskillelse af Drag og Dobbeltklik**:
   - I `DiagramCanvas` evalueres dobbeltklik p.t. på `ButtonPressed`. Hvis en bruger klikker på en node for at vælge den og inden for 500 ms trykker musen ned for at trække (`ButtonPressed` ➔ `CursorMoved`), må trækket ALDRIG afbrydes eller konverteres til et dobbeltklik.
   - Dobbeltklik (både på node og tomt lærred) skal kun bekræftes, når brugeren udfører to på hinanden følgende klik med lav bevægelse (`drag_dist < 4.0 px`), valideret ved `ButtonReleased` (eller hvor et påbegyndt træk med `drag_dist >= 4.0 px` annullerer dobbeltklik-tilstanden).
2. **Telemetri for Lynoprettelse & Canvas Interaktion**:
   - `Message::CanvasDoubleClicked(x, y)` i `src/ui/app.rs` skal logge eksplicit via `diagnostics::log_info("canvas", &format!("Lynoprettede begreb via dobbeltklik på ({:.1}, {:.1})", x, y))` så fremtidig fejlfinding har direkte evidens for auto-oprettede begreber.
3. **Håndtering af Annulleret Hurtigoprettelse**:
   - Hvis en bruger har trigget et dobbeltklik på tomt lærred, og den resulterende inline editor forlades/annulleres med tom foretrukken term, ryddes det midlertidigt oprettede begreb op fra projekt og lærred, så der ikke efterlades tomme spøgelsesbegreber ("Nyt begreb").

## 📋 Acceptance Criteria
- [ ] **AC1 - Drag afbrydes ikke af forudgående klik**: Hvis en bruger klikker på en node (eller tomt lærred) og inden for 500 ms trykker ned og trækker musen (>4 px), udføres trækket normalt. Noderne flyttes, og hverken `CanvasDoubleClicked` eller `GraphNodeDoubleClicked` affyres.
- [ ] **AC2 - Dobbeltklik på tomt lærred ved reelt dobbeltklik**: To på hinanden følgende klik på tomt lærred uden bevægelse (<4 px) inden for 500 ms udløser `CanvasDoubleClicked`.
- [ ] **AC3 - Dobbeltklik på node ved reelt dobbeltklik**: To på hinanden følgende klik på en node uden bevægelse (<4 px) inden for 500 ms udløser `GraphNodeDoubleClicked` / `InfoNodeDoubleClicked` og sætter tastaturfokus i navnefeltet (Task 073 adfærd bevares).
- [ ] **AC4 - Telemetri-logning ved lynoprettelse**: Oprettelse af et begreb via dobbeltklik på lærredet logges i in-memory logbufferen med koordinater.
- [ ] **AC5 - Automatisk oprydning ved annullering af tom lyn-node**: Hvis et lynoprettet begreb forlades eller annulleres uden at brugeren har indtastet en gyldig foretrukken term, fjernes det midlertidige begreb fra projektet.
- [ ] **AC6 - Regression Gauntlet**: Samtlige canvas- og accepttests i workspacet passerer 100% grønt med zero clippy warnings.

## 🚫 Must NOT
- Må IKKE ødelægge eksisterende multi-selection marquee eller bulk drag (Task 040 & Task 067).
- Må IKKE forringe dobbeltklik-følsomheden ved bevidst dobbeltklik (standard 500 ms interval og 20 px rumlig tolerance).
- Må IKKE udføre synkronskrivning under træk.
- Må IKKE udføre remote push eller destruktive git handlinger.

## 📝 Revisions
- 2026-10-06: Oprettet opgavepakke efter opdaget race-condition mellem hurtig node-udvælgelse, nodetræk og lærreds-dobbeltklik.

## 🧪 Verifikation
- `cargo test --test acceptance test_task_077_`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets`
- `cargo fmt --check`
