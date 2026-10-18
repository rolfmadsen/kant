---
type: Task Package
title: "Task 085: Canvas Studio Decomposition"
description: "Dekomponering af monolitten src/ui/diagram_canvas.rs (2.568 linjer) i modulære canvas-komponenter under 800 linjer og stramning af ratchet spærrehage"
status: active
generated: { by: process:antigravity-task-init, at: "2026-10-18T18:00:30Z" }
tags: [refactor, canvas, decomposition, modularity, architecture, ratchet]
scope:
  - "src/ui/diagram_canvas.rs"
  - "src/ui/canvas/**"
  - "src/ui/mod.rs"
  - "gauntlet.toml"
  - "tests/architecture_test.rs"
test_refs:
  - "tests/features/canvas_acceptance_test.rs"
  - "tests/architecture_test.rs"
---

# Task 085: Canvas Studio Decomposition

**Status**: `ACTIVE`  
**Intent**: `🔄 REFACTOR`  
**Oprettet**: `2026-10-18`  

## 🎯 Formål
Dekomponere Kants største visuelle monolit `src/ui/diagram_canvas.rs` (2.568 linjer) i overskuelige, modulære enheder under `src/ui/canvas/`, så alle filer overholder loftet på højst 800 linjer:
1. Udskille koordinattransformation, zoom og pan til et dedikeret viewport-modul.
2. Isolere rendering af begrebsnoder, relationer og minimap i separate renderer-moduler.
3. Flytte mus- og tastaturinteraktion (drag-select, multi-node flytning, forbindelsesoprettelse) til et interaktionsmodul.
4. Reducere eller helt fjerne undtagelsen for `diagram_canvas.rs` i `gauntlet.toml`.

## 📋 Acceptance Criteria
- [ ] **AC1 (Viewport & Geometri Modul)**: Opret `src/ui/canvas/viewport.rs` ($\le 400$ linjer), der indkapsler koordinatsystem, zoom-niveauer, pan-offset og synlige skærm-bounds.
- [ ] **AC2 (Node & Edge Renderers)**: Opret `src/ui/canvas/node_renderer.rs` ($\le 600$ linjer) til tegning af FDA begrebskasser og `src/ui/canvas/edge_renderer.rs` ($\le 500$ linjer) til tegning af relationer, pile og multipliciteter.
- [ ] **AC3 (Minimap & Interaktion)**: Udskil minimap rendering til `src/ui/canvas/minimap.rs` ($\le 300$ linjer) og mus/tastatur event-håndtering til `src/ui/canvas/interaction.rs` ($\le 600$ linjer).
- [ ] **AC4 (Canvas Shell & Ratchet Opdatering)**: Hovedmodulet `src/ui/canvas/mod.rs` orkestrerer Iced Canvas `Program`-traitet under 800 linjer, og undtagelsen for `src/ui/diagram_canvas.rs` slettes eller strammes til under 800 linjer i `gauntlet.toml`.
- [ ] **AC5 (Regression & Verifikation)**: Samtlige canvas-accepttests i `tests/features/canvas_acceptance_test.rs` og `tests/acceptance.rs` består uændret, og `xgauntlet check-architecture` godkender modulet uden overtrædelser.

## 🚫 Must NOT
- Må IKKE ændre eksisterende canvas-adfærd, rendering af FDA diagrammer eller tastaturgenveje (frosne assertions).
- Må IKKE oprette nye moduler over 800 linjer.
- Må IKKE lempes på spærrehager i `gauntlet.toml`.

## 📝 Revisions
- 2026-10-18: Oprettet opgavepakke til dekomponering af diagram_canvas.rs.

## 🧪 Verifikation
- `cargo test --test canvas_acceptance`
- `cargo test --test architecture_test`
- `xgauntlet check-architecture`
- `xgauntlet verify`
- `xgauntlet check-spec -t 085-canvas-studio-decomposition`
