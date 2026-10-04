---
type: Task Package
title: "Task 074: Palet-valg Nulstiller og Synkroniserer Canvas Node-markering"
description: "Sikre at markering af en klasse eller enumeration i venstre palet rydder tidligere valgte canvas-noder og kun markerer den nye node hvis den findes på lærredet."
status: done
generated: { by: process:antigravity-task-init, at: "2026-10-04T22:38:00Z" }
tags: [ui, palette, canvas, selection, synchronization, information-model]
---

# Task 074: Palet-valg Nulstiller og Synkroniserer Canvas Node-markering

**Status**: `DONE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-10-04`

## 🎯 Formål
Løse desynkronisering mellem palet-valg og lærredsmarkering i Informationsmodellen:
1. Når en node A er markeret på lærredet, og brugeren klikker på en anden klasse B i paletten:
   - Hvis klasse B er på lærredet som node B, skal node A afmarkeres og node B markeres (`selected_info_graph_node_ids = {node_B}`).
   - Hvis klasse B ikke er på lærredet, skal node A afmarkeres og ingen noder være markerede (`selected_info_graph_node_ids = {}`).
2. Samme konsistente adfærd når en enumeration vælges i paletten.
3. Sørge for at `selected_info_graph_node_id` og `selected_info_graph_node_ids` altid forbliver 100% konsistente i samtlige mutationer og valg.

## 📋 Acceptance Criteria
- [x] **AC1 - Palet-valg af klasse på canvas**: Valg af klasse B i paletten afmarkerer node A og markerer node B på lærredet.
- [x] **AC2 - Palet-valg af klasse udenfor canvas**: Valg af klasse C i paletten afmarkerer node A så lærredet har 0 markerede noder.
- [x] **AC3 - Palet-valg af enumeration**: Valg af en enumeration i paletten afmarkerer enhver klasse på lærredet og markerer enumerationens node hvis den er på lærredet.
- [x] **AC4 - Konsistent tilstandsforvaltning**: `selected_info_graph_node_id` og `selected_info_graph_node_ids` synkroniseres i samtlige CRUD- og udvælgelseshandlinger.
- [x] **AC5 - Regression Gauntlet**: Alle workspace tests og clippy lints forbliver 100% grønne.

## 🚫 Must NOT
- Må IKKE efterlade forældede noder i `selected_info_graph_node_ids`.
- Må IKKE ødelægge multi-node selektion via lærredet.

## 📝 Revisions
- 2026-10-04: Oprettet opgavepakke efter opdaget fejl hvor tidligere valgt canvas-node forblev markeret ved palet-valg.
- 2026-10-04: Implementeret `set_selected_info_graph_node` og opdateret alle palette-, canvas- og diagramhandlere. Bekræftet via accepttest `test_task_074`. Opgave afsluttet som DONE.

## 🧪 Verifikation
- `cargo test test_task_074`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
