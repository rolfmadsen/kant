---
type: Task Package
title: "Task 077: Adskillelse af Canvas Drag og Dobbeltklik samt Node-oprettelse Telemetri"
description: "DiagramCanvas skal kun udløse dobbeltklik ved to stillestående klik på samme mål, aldrig afbryde et påbegyndt træk, logge lynoprettelser og rulle tomme lynoprettede begreber tilbage."
status: done
generated: { by: process:antigravity-task-init, at: "2026-10-06T19:35:00Z" }
tags: [canvas, drag-and-drop, double-click, mouse, telemetry, concept-model, information-model]
---

# Task 077: Adskillelse af Canvas Drag og Dobbeltklik samt Node-oprettelse Telemetri

**Status**: `DONE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-10-06`
**Scope**: `src/ui/diagram_canvas.rs`, `src/ui/app.rs`, `tests/acceptance.rs`
**Blokeret af**: Task 076 (frysningen er hovedårsagen til fantom-dobbeltklik; uden 076 kan 077 ikke verificeres manuelt)

## 🎯 Formål
Robustgøre `DiagramCanvas` museinteraktion så hurtigt træk efter valg ikke mistolkes som dobbeltklik, forhindre utilsigtede lynoprettelser af noder samt rulle tomme begreber tilbage:
1. **Adskillelse af Drag og Dobbeltklik**: Dobbeltklik evalueres kun ved to stillestående klik på samme mål (<4 skærm-px) ved `ButtonReleased`. Et påbegyndt træk (bevægelse ≥ 4 skærm-px) annullerer altid dobbeltklik-tilstanden og gennemfører trækket.
2. **Kildebevidst Telemetri**: `Message::CanvasDoubleClicked` og `CreateConceptAtCenter` logger eksplicit oprettelse af begreber med id og kilde.
3. **Rollback af tomme lynoprettede begreber**: Hvis en bruger lynopretter et begreb ved en fejl og lukker/annullerer editoren uden at indtaste et navn, fjernes det midlertidige begreb fra projektet.

## 🔬 Problem (verificeret i kode, v0.8.0)
`DiagramCanvas` ([diagram_canvas.rs:755–785](../src/ui/diagram_canvas.rs)) afgør dobbeltklik ved **andet `ButtonPressed`**:
- Kriterium i dag: ≤ 500 ms og ≤ 20 px (skærm) fra *forrige tryk* — uanset hvad der blev ramt første gang.
- Ved match ryddes `dragging_node`/`bulk_drag` (Task 073), og beskeden sendes straks.

Konsekvenser:
1. **Klik-for-at-vælge, tryk-for-at-trække** (inden for 500 ms) åbner editoren på noden (`GraphNodeDoubleClicked`) i stedet for at trække.
2. **Forskellige mål**: klik på node, derefter tryk på tomt lærred ≤ 20 px væk → `CanvasDoubleClicked` → **ny node "Nyt begreb"**.
3. **Ctrl-klik** (multiselektion) sætter også `last_click`, så Ctrl+klik efterfulgt af almindeligt tryk kan blive dobbeltklik.
4. **Inkonsistente enheder**: dobbeltklik-afstand måles i skærm-px, træk-tærsklen (`drag_dist >= 4.0`) i verdens-koordinater → ved zoom 0,25 er 4 verdens-px = 1 skærm-px.
5. `CanvasDoubleClicked` logger intet, så lynoprettelser er usynlige i diagnostikken.
6. `CancelConceptEdit` ([app.rs:4068](../src/ui/app.rs)) lukker kun editoren. Det lynoprettede begreb er allerede gemt (autosave) og broadcastet (`ConceptAdded`), så det bliver liggende som "Nyt begreb".
7. `CreateConceptAtCenter` (app.rs ~5676) genbruger `CanvasDoubleClicked`, så telemetri i den handler ville fejlagtigt sige "dobbeltklik".

## 🏗️ Design (bindende)
### A. Pending double-click i `DiagramCanvasState`
```rust
enum ClickTarget { Node(NodeId), Empty }
struct ClickRecord { position: Point /* skærm */, time: Instant, target: ClickTarget }
pending_double: Option<ClickTarget>   // ny
press_screen_pos: Option<Point>       // ny
```
- **ButtonPressed (venstre, uden Ctrl/Cmd)**: beregn `target`. Kandidat hvis `last_click` har samme `target`, ≤ 500 ms og ≤ 20 px skærm. Kandidat → `pending_double = Some(target)`. **Start træk/marquee præcis som i dag** — intet ryddes.
- **ButtonPressed med Ctrl/Cmd**: `last_click = None`, `pending_double = None` (Ctrl-klik deltager aldrig i dobbeltklik).
- **CursorMoved**: hvis afstand fra `press_screen_pos` ≥ 4 skærm-px → `pending_double = None`, `last_click = None`. Herefter almindelig drag-logik (Task 067 delta-filter uændret).
- **ButtonReleased**:
  - `pending_double.take()` er `Some` og bevægelse < 4 skærm-px → ryd `bulk_drag`/`dragging_node`/`marquee` **uden** `on_drag_released` og **uden** single-click-sideeffekter (selektionskollaps til leader, "klik på tomt rydder selektion"), sæt `last_click = None` og publicér `on_node_double_clicked(id)` eller `on_canvas_double_clicked(x, y)` (verdens-koordinater fra *første* klik).
  - Ellers: nuværende adfærd. Efter et stillestående klik registreres `last_click = Some(ClickRecord { .. })`.
- **Tærskler**: én konstant `CLICK_SLOP_PX: f32 = 4.0` i skærm-px for både drag-start og dobbeltklik-annullering. Eksisterende `drag_dist >= 4.0` (verden) erstattes af skærm-baseret måling.

### B. Telemetri (kildebevidst)
- `CanvasDoubleClicked` deler logik med `CreateConceptAtCenter` via `fn quick_create_concept_at(&mut self, x, y, source: QuickCreateSource)`.
- Log: `log_info("canvas", "Lynoprettede begreb {id} via {dobbeltklik|værktøjslinje} på ({x:.1}, {y:.1})")`.
- `GraphNodeDoubleClicked` og `InfoNodeDoubleClicked`: `log_info("canvas", "Dobbeltklik på node {id} – åbner egenskaber")`.

### C. Rollback af tomt lynoprettet begreb
- Nyt felt: `pending_quick_create: Option<Uuid>` (concept id), sættes i `quick_create_concept_at`.
- Ryddes (uden rollback) når `SaveConcept` lykkes for det id.
- Rollback når editoren lukkes uden gyldig save og `preferred_term` er tom (trimmet). Gælder mindst: `CancelConceptEdit`, Escape (app.rs ~4786 → `CancelConceptEdit`), valg af anden node/tomt lærred, fanebyt.
- Implementeres centralt: `fn close_concept_editor(&mut self)`, som alle ovenstående stier kalder.
- Rollback skal: fjerne begreb + node fra projekt/diagram (genbrug eksisterende slette-sti, app.rs ~4000), broadcaste `ModelMutation::ConceptDeleted(id)`, nulstille `selected_graph_node_id(s)` og kalde `trigger_autosave()`.
- Hvis brugeren har udfyldt en term, rulles der aldrig tilbage, heller ikke ved annullering.

## 📋 Acceptance Criteria
- [x] **AC1 – Klik + træk er et træk**: Klik (tryk+slip) på node A, derefter straks tryk på A og flyt ≥ 4 skærm-px → `on_nodes_moved` udsendes; hverken `on_node_double_clicked` eller `on_canvas_double_clicked`. Gælder også for multiselektion (2 noder flyttes).
- [x] **AC2 – Reelt dobbeltklik på tomt lærred**: To stillestående klik på tomt lærred (≤ 500 ms, ≤ 20 px) → præcis én `on_canvas_double_clicked`, udsendt ved **andet slip**.
- [x] **AC3 – Reelt dobbeltklik på node (Task 073 bevares)**: To stillestående klik på node A → præcis én `on_node_double_clicked(A)` ved andet slip, ingen `on_drag_released`, ingen ændring af nodeposition; `test_task_073_*` passerer uændret.
- [x] **AC4 – Samme mål kræves**: Klik på node A efterfulgt af klik på tomt lærred (eller node B) inden for 500 ms/20 px → intet dobbeltklik.
- [x] **AC5 – Ctrl-klik deltager ikke**: Ctrl+klik på A efterfulgt af klik på A → intet dobbeltklik.
- [x] **AC6 – Zoom-uafhængig tærskel**: Ved zoom 0,25 og 2,0 starter træk ved samme skærm-afstand (4 px).
- [x] **AC7 – Telemetri**: Lynoprettelse via dobbeltklik og via `CreateConceptAtCenter` logger hver sin kilde med begrebs-id og koordinater; node-dobbeltklik logges.
- [x] **AC8 – Rollback**: `CanvasDoubleClicked` → `CancelConceptEdit` med tom term → begrebet og noden findes ikke i projektet, og autosave er udløst. Med udfyldt term → begrebet bevares.
- [x] **AC9 – Regression Gauntlet**: Task 040 (bulk drag/marquee), Task 067 (delta-filter, drop-commit, `test_*drag*`), Task 073 tests og `test_graph_canvas_double_click_events` passerer; `cargo clippy --workspace --all-targets -- -D warnings` grøn.

## 🚫 Must NOT
- Må IKKE afbryde et påbegyndt træk på grund af et forudgående klik.
- Må IKKE ændre dobbeltklik-vinduet (500 ms / 20 px skærm) for bevidste dobbeltklik.
- Må IKKE udsende `on_drag_released` (og dermed autosave) for et dobbeltklik uden bevægelse.
- Må IKKE logge pr. `CursorMoved` (Task 067).
- Må IKKE rulle et begreb tilbage, som brugeren har givet en term.
- Må IKKE udføre `git push` eller destruktive git-handlinger.

## ⚠️ Kendt restrisiko (accepteret, dokumenteres)
Iced-musehændelser har intet tidsstempel; `Instant::now()` måles ved behandling. Hvis UI-tråden blokeres af anden årsag (se Task 076 "Uden for scope"), kan to *stillestående* klik under frysningen stadig blive læst som dobbeltklik. AC8-rollback og AC7-telemetri begrænser skaden og gør det synligt.

## ⛔ Uden for scope
- Informationsmodellens `CreateInformationClassAt` (tomt-lærred-dobbeltklik i fane 3) får samme canvas-adfærd via `DiagramCanvas`, men rollback af tomme klasser er ikke en del af denne task.

## 🧪 RED-hypoteser
1. `test_task_077_click_then_drag_moves_nodes` (unit i `diagram_canvas.rs`, mønster som ~2260–2324 med `AtomicUsize`-tællere) fejler i dag: andet tryk udløser `on_node_double_clicked`, og `on_nodes_moved` tælles 0.
2. `test_task_077_double_click_requires_same_target` fejler i dag: node→tomt lærred udløser `on_canvas_double_clicked`.
3. `test_task_077_double_click_fires_on_release` fejler i dag: beskeden udsendes ved tryk.
4. `test_task_077_cancel_empty_quick_create_rolls_back` (acceptance) fejler i dag: begrebet findes stadig efter `CancelConceptEdit`.

## 📝 Revisions
- 2026-10-06: Oprettet opgavepakke efter opdaget race-condition mellem hurtig node-udvælgelse, nodetræk og lærreds-dobbeltklik.
- 2026-10-06: Skærpet efter review: tilstandsmaskine for pending double-click, krav om samme mål, Ctrl-undtagelse, skærm-px-tærskel, kildebevidst telemetri (`CreateConceptAtCenter`), centraliseret rollback inkl. broadcast, dokumenteret restrisiko og afhængighed af Task 076.

## 🧪 Verifikation
- `cargo test test_task_077_`
- `cargo test --test acceptance test_task_073_`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
- `xgauntlet check-spec -t 077`
