---
type: Task Package
title: "Task 092: Palette Search Alignment & Header Cleanup"
description: "Fjerner overflødig '+ Fra begreb...' dropdown fra Informationsmodellens venstrepalet og centrerer søgefeltets tekst og ikon vertikalt på tværs af Begrebsdiagram og Informationsmodel."
status: active
generated: { by: process:antigravity-task-init, at: "2026-10-26T21:36:00Z" }
tags: [usability, ui, palette, alignment, cleanup, concept-model, information-model]
scope:
  - "src/features/concept_model/ui/palette.rs"
  - "src/features/information_model/ui/class_list.rs"
  - "src/ui/information_model_view.rs"
test_refs:
  - "tests/features/palette_search_and_header_cleanup_acceptance_test.rs"
---

# Task 092: Palette Search Alignment & Header Cleanup

**Status**: `ACTIVE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-10-26`

## 🎯 Formål
Gøre venstrepaletten på Begrebsdiagram (Fane 2) og Informationsmodel (Fane 3) æstetisk strømlinet og symmetrisk ved at:
1. Fjerne den forældede og overflødige `+ Fra begreb...` pick_list fra klasselistens paletheader, da begrebssporing styres autoritativt i Egenskabspanelets Begrebssporing (FDA).
2. Justere søgeinputtets padding og højde fra flad 5px til balanceret `[7, 9]`, så `🔍` og placeholder-teksten centreres optisk perfekt på den vertikale akse uden at klemme sig mod toppen.

## 📋 Acceptance Criteria
- [ ] **AC1 - Fjernelse af '+ Fra begreb...' pick_list**: `+ Fra begreb...` pick_listen er fjernet fra Informationsmodellens venstrepalet (`class_list.rs`), og unødvendig filtreringslogik i `information_model_view.rs` er saneret.
- [ ] **AC2 - Optisk lodret centrering af søgefeltet i Begrebspaletten (Fane 2)**: Søgefeltet i Begrebspaletten (`palette.rs`) anvender balanceret vertikal padding `[7, 9]` (`SEARCH_INPUT_PADDING`), så `🔍` ikonet og teksten er visuelt centreret i feltet.
- [ ] **AC3 - Optisk lodret centrering af søgefeltet i Klasselisten (Fane 3)**: Søgefeltet i Klasselisten (`class_list.rs`) anvender balanceret vertikal padding `[7, 9]` (`SEARCH_INPUT_PADDING`), så `🔍` ikonet og teksten er visuelt centreret i feltet.
- [ ] **AC4 - Intakt Begrebssporing og Egenskabspanel**: Egenskabspanelets "Begrebssporing (FDA)" bevarer fuld kapacitet til at tilknytte og aggregere multiple begreber for klasser og attributter.

## 🚫 Must NOT
- Må IKKE fjerne eller forringe begrebssporingen (`concept_ids`) i `InformationClass` eller `Attribute`.
- Må IKKE bryde tastaturnavigation, combobox-navigation (`Pil Op` / `Pil Ned` / `Enter`) eller søgefiltrering.
- Må IKKE ændre eksisterende diagram- og layoutinvarianter på canvas.

## 📝 Revisions
- 2026-10-26: Oprettet opgavepakke efter brugerfeedback vedr. ucentreret søgeikon/tekst og forslag om oprydning i overflødig dropdown.

## 🧪 Verifikation
- `xgauntlet check-spec -t 092`
- `cargo test --test palette_search_and_header_cleanup_acceptance`
- `xgauntlet verify -t 092`
