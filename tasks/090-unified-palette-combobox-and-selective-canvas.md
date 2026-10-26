---
type: Task Package
title: "Task 090: Unified Palette Combobox and Selective Canvas Placement"
description: "Selektiv placering af begreber på canvas (ingen auto-dumping fra begrebslisten eller Excel), combobox-søgenavigation med piletaster og Enter i venstrepaletten, automatisk centrering på valgt node ved bevaret zoom, samt placering i viewport-centrum for u-placerede elementer på tværs af Begrebsdiagram og Informationsmodel."
status: done
generated: { by: process:antigravity-task-init, at: "2026-10-26T20:12:00Z" }
tags: [usability, canvas, palette, combobox, keyboard-navigation, concept-model, information-model, ergonomics]
scope:
  - "src/features/concept_model/**"
  - "src/features/information_model/**"
  - "src/features/model/**"
  - "src/ui/**"
  - "tests/features/concept_palette_combobox_acceptance_test.rs"
test_refs:
  - "tests/features/concept_palette_combobox_acceptance_test.rs"
---

# Task 090: Unified Palette Combobox and Selective Canvas Placement

**Status**: `DONE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-10-26`

## 🎯 Formål
Forbedre diagram-ergonomi og metodisk overensstemmelse med FDA Modelreglerne ved at:
1. Adskille katalogregistrering i Begrebslisten fra diagramlærredet, så nye eller importerede begreber ikke automatisk dumpes som diagramnoder, men forbliver i paletten som u-placerede.
2. Forvandle venstrepalettens søgefelt til en ægte combobox med piletastnavigation (`Pil Op` / `Pil Ned`), automatisk markørplacering ved åbning via `Ctrl+B` eller ikon, samt totrins Escape-rydning/lukning.
3. Tilbyde lynhurtig tastaturdrevet lærredsinteraktion med `Enter`:
   - Hvis elementet findes på lærredet: Vælger noden, overfører fokus til canvas og centrerer viewporten på noden ved bevaret zoomniveau.
   - Hvis elementet er u-placeret: Opretter noden direkte i centrum af den synlige viewport (med 20px gitter-snapping), markerer den og overfører fokus til canvas.
4. Harmonisere denne adfærd på tværs af både **2. Begrebsdiagram** og **3. Informationsmodel** via en fælles centrerings-metode på `CanvasViewport`.

## 📋 Acceptance Criteria
- [x] **AC1 - Selektiv Lærredsplacering for Begreber**: Oprettelse af begreber i Begrebslisten (`ModelProject::add_concept`) eller import via Excel/CSV tilføjer IKKE længere automatisk noder til `ConceptGraph`. Begreberne forbliver u-placerede i Begrebspaletten (`✓` vises ikke, `+` vises).
- [x] **AC2 - Autofokus i Palettens Søgefelt (`Ctrl+B` / Ikon)**: Både i 2. Begrebsdiagram og 3. Informationsmodel sættes tastaturfokus direkte i søgefeltet (`concept_model_search_input` / `info_class_search_input`), når venstrepaletten åbnes via `Ctrl+B` eller `◧` klik.
- [x] **AC3 - Combobox Piletastnavigation i Paletten**: Når søgefeltet eller paletten er i fokus, navigerer `Pil Ned` og `Pil Op` cyklisk igennem de filtrerede elementer med en synlig fokusmarkør (`is_active` highlight).
- [x] **AC4 - Enter-Centrering på Placeret Node**: Tryk på `Enter` på et element, der allerede er placeret på canvas, vælger elementet, overfører fokus til canvas og opdaterer `CanvasViewport` med bevaret zoom, så elementets centrum er centreret i det synlige arbejdsområde.
- [x] **AC5 - Enter-Placering af U-Placeret Element**: Tryk på `Enter` på et element, der endnu IKKE er på canvas, tilføjer det øjeblikkeligt i centrum af den synlige viewport (verdenskoordinater under hensyntagen til zoom og pan med 20px magnetisk gitter-snapping), markerer elementet, opdaterer palettens status til `✓` og overfører fokus til canvas.
- [x] **AC6 - Escape-Håndtering i Søgefeltet**: Tryk på `Escape` i palettens søgefelt rydder aktiv søgetekst, hvis feltet indeholder tekst. Hvis søgefeltet er tomt, lukkes paletten (`ToggleLeftSidebar`) og fokus sendes til canvas.
- [x] **AC7 - Fælles Harmoniseret CanvasViewport Centrerings-Motor**: `CanvasViewport` indeholder en dedikeret metode `center_on_rect(x, y, w, h, screen_center)`, der understøtter både Fane 2 (Begrebsmodel) og Fane 3 (Informationsmodel) med bevarelse af `zoom`.

## 🚫 Must NOT
- Må IKKE fjerne begreber eller klasser fra modellen, når de fjernes fra eller ikke optræder på canvas.
- Må IKKE ændre eksisterende diagrammer, hvor noder allerede er gemt med persistente koordinater.
- Må IKKE overtræde zoom-grænser (0.25 - 1.50) eller forskyde pan uden for gyldige koordinater.
- Må IKKE bryde eksisterende drag-and-drop eller 1-klik `+` knap i paletten (Task 064 bevares).

## 📝 Revisions
- 2026-10-26: Oprettet opgavepakke efter usability- og arkitektursparring om combobox-navigation, selektiv lærredsplacering og harmonisering mellem Begrebsdiagram og Informationsmodel.
- 2026-10-26: Implementeret og verificeret med `concept_palette_combobox_acceptance_test` og 100% grøn testsuite. Opfyldt AC1-AC7.

## 🧪 Verifikation
- `xgauntlet check-spec -t 090`
- `cargo test --test concept_palette_combobox_acceptance`
- `xgauntlet verify`
