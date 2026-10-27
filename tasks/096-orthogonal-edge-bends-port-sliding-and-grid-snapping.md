---
type: Task Package
title: "Task 096: Ortogonal Edge Segment-Dragging, Bends, Grid Snapping og Dynamisk Port-Sliding"
description: "Gør relationer på canvas interaktivt redigerbare med yEd-lignende kontrol over ortogonale linjestykker, frie knækpunkter, grid-snapping, forankrede bends og dynamisk port-sliding på tværs af Begrebs- og Informationsmodellen."
status: done
generated: { by: process:antigravity-task-init, at: "2026-10-27T22:36:00Z" }
tags: [canvas, edge-routing, orthogonal, bends, port-sliding, grid-snapping, usability, concept-model, information-model, persistence]
scope:
  - "src/features/concept_model/mod.rs"
  - "src/features/information_model/diagram.rs"
  - "src/features/model/decomposed.rs"
  - "src/ui/canvas/mod.rs"
  - "src/ui/canvas/interaction.rs"
  - "src/ui/canvas/edge_renderer.rs"
  - "src/ui/edge_router/types.rs"
  - "src/ui/edge_router/geometry.rs"
  - "src/ui/edge_router/channels.rs"
  - "src/ui/edge_router/router.rs"
  - "src/ui/app.rs"
  - "src/ui/inspector_panel.rs"
  - "docs/adr/005-orthogonal-edge-routing-and-port-bus.md"
  - "Cargo.toml"
test_refs:
  - "tests/features/edge_manual_routing_acceptance_test.rs"
---

# Task 096: Ortogonal Edge Segment-Dragging, Bends, Grid Snapping og Dynamisk Port-Sliding

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-10-27`

## 🎯 Formål
Gøre relationer på canvas interaktivt redigerbare med yEd-lignende præcisionskontrol over ortogonale linjestykker og frie knækpunkter (*bends*), gældende for både Begrebsmodellen (Fane 2) og Informationsmodellen (Fane 3). 

Automatisk autorouting bevares som standardtilstand, mens relationer kan overstyres med eksplicitte knækpunkter. Eksisterende knækpunkter forankres rumligt på canvas ved node-flytninger uden tavs nulstilling. Terminale segmenter understøtter glidende portforbindelse (*port-sliding*) langs nodens kant for at eliminere mikrobøjninger og overflødige L-knæk.

---

## 📋 Acceptance Criteria

- [x] **AC1 - Domæne-neutral BendPoint Datamodel:** Introducer `BendPoint { pub x: f32, pub y: f32 }` i domænelaget (`src/features/concept_model/mod.rs`) uden afhængighed til Iced. Både `DiagramEdge` og `ClassDiagramEdge` udvides med `bends: Option<Vec<BendPoint>>`, og `CanvasEdge`-trækket i `src/ui/canvas/mod.rs` eksponerer `bends(&self) -> Option<&[BendPoint]>`.
- [x] **AC2 - Deterministisk Dekomponeret Persistens:** `bends` serialiseres deterministisk i `.kant/relations/<uuid>.json` for både `DecomposedRelation::ConceptRelation` og `DecomposedRelation::ClassRelation` (`skip_serializing_if = "Option::is_none"` når feltet er tomt eller None).
- [x] **AC3 - Ortogonal Routing med Bends:** `EdgeRouter::route_edges` genererer 90-graders Manhattan-stier, der passerer gennem samtlige mellemliggende knækpunkter $[P_1, \dots, P_k]$ med ortogonal docking ved kilde- og målporte jf. FDA Modelreglerne og ADR 005.
- [x] **AC4 - Dynamisk Port-Sliding:** For relationer med knækpunkter tilpasser kilde- og målportens ankerpunkt sig det terminale segments position: portens akse glider langs nodens over-/under- eller sidekant, så den flugter direkte med segmentet (afgrænset inden for nodens kasse minus hjørnemargin på 8px).
- [x] **AC5 - Zonedrevet Portskift & Syntetiseret Tilslutning:** Hvis et terminalt segment trækkes forbi et hjørne, skifter porten automatisk til tilstødende zone. Forbindelsen mellem port og $P_1$ (hhv. $P_k$ og målport) syntetiseres on-the-fly af routeren uden at forurene `manual_bends` med redundante hjørnepunkter.
- [x] **AC6 - Grid Snapping ved Drag:** Flytning af knækpunkter og segment-træk snappes til canvas-griddet (`GRID_SIZE = 20.0`) og respekterer canvas `snap_to_grid` indstillingen.
- [x] **AC7 - Ortogonal Segment Dragging:** Træk i et horisontalt segment flytter udelukkende dets Y-koordinat; træk i et vertikalt segment flytter udelukkende dets X-koordinat. Nabosegmenter forlænges/forkortes uden at bryde 90° vinkler.
- [x] **AC8 - Segment-Split på Rette Linjer:** Vinkelret træk i et lige linjesegment uden bends splitter segmentet i tre og opretter to nye knækpunkter på griddet.
- [x] **AC9 - Forankrede Bends ved Node Drag:** Flytning af noder ændrer ikke eksisterende knækpunkters absolutte canvas-koordinater. Forbindelserne forlænges og tilpasses dynamisk. Hvis en node trækkes hen over $P_1$ i segmentets retning, absorberes punktet for at forhindre baglæns sløjfer.
- [x] **AC10 - Collinear Pruning & Mikrosegment-kollaps ved Release:** Ved `MouseRelease` slettes redundante kollineære bends ($P_{i-1}, P_i, P_{i+1}$ på samme rette linje inden for $\epsilon = 0.5\text{px}$), og mikrosegmenter under 4px absorberes.
- [x] **AC11 - Inspector Nulstilling & Tastatursletning:** Egenskabspanelet (Inspector) udstiller knappen `[Gendan automatisk linjeføring]` for relationer med manuelle bends. Dobbeltklik på et bend eller `Delete` / `Backspace` fjerner det pågældende knækpunkt.
- [x] **AC12 - Drop-Commit Autosave Invariant:** Under aktiv trækning opdateres kun den flygtige canvas-tilstand via `Action::request_redraw()`. Autosave og disk-skrivning sker udelukkende ved drop-commit (`ButtonReleased`).
- [x] **AC13 - Isoleret Acceptancetest-suite:** Opret en dedikeret testfil `tests/features/edge_manual_routing_acceptance_test.rs` (registreret i `Cargo.toml`), der dækker port-sliding, segment dragging, grid snapping, forankring under node-flytning og persistens.

---

## 🚫 Must NOT
- Må IKKE importere `iced` eller GUI-typer i domænelaget (`src/features/model/`, `src/features/concept_model/`, `src/features/information_model/`).
- Må IKKE udføre disk I/O eller autosave under aktiv musetræk (`CursorMoved`).
- Må IKKE bryde FDA pilehoveddocking: Terminale segmenter skal altid ramme nodens yderkant vinkelret, så trekant- eller diamantmarkøren slutter plant mod kanten.
- Må IKKE foretage tavs nulstilling af `bends` ved node-flytning.
- Må IKKE overskride 1.000 linjers loftet for testfiler (`canvas_acceptance_test.rs` må ikke overfyldes).
- Må IKKE foretage remote publication handlinger (`git push`).

---

## 📝 Revisions
- 2026-10-27: Oprettet opgavepakke efter sparringsprocedure med skærpede arkitekturinvarianter (BendPoint i domænet, fuld støtte for både Begrebs- og Klassediagram, Inspector-nulstilling og isoleret testfil).
- 2026-10-27: Implementeret og verificeret alle 13 acceptkriterier med grøn testsuite, fuld arkitekturoverholdelse og tovejs Begrebs- og Informationsmodel UI support.

---

## 🧪 Verifikation
- `xgauntlet check-spec -t 096`
- `cargo test --test edge_manual_routing_acceptance`
- `cargo clippy --all-targets -- -D warnings`

