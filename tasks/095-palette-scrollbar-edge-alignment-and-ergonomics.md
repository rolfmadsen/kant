---
type: Task Package
title: "Task 095: Palette Scrollbar Edge Alignment and Ergonomics"
description: "Placerer den vertikale scrollbar helt ude ved højrekanten på venstrepaletten for Begrebsdiagram og Informationsmodel, etablerer en dedikeret sikkerhedszone til handlingsikonerne (+, ✓, 🗑️), og øger scrollbarens bredde til 6px."
status: done
generated: { by: process:antigravity-task-init, at: "2026-10-27T22:14:00Z" }
tags: [usability, ui, palette, scrollbar, layout, styling, concept-model, information-model, ergonomics]
scope:
  - "src/features/concept_model/ui/palette.rs"
  - "src/features/information_model/ui/class_list.rs"
  - "src/ui/theme.rs"
test_refs:
  - "tests/features/palette_scrollbar_edge_alignment_acceptance_test.rs"
  - "tests/features/palette_scrollbar_clearance_acceptance_test.rs"
---

# Task 095: Palette Scrollbar Edge Alignment and Ergonomics

**Status**: `DONE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-10-27`

## 🎯 Formål
Forbedre scrollbar-ergonomien og layoutet i venstrepaletten for både Begrebsdiagram og Informationsmodel: Scrollbaren flyttes helt ud, så den flugter med panelets højre kant, scrollbarens bredde øges til 6px for nem interaktion med musen, og der etableres en dedikeret sikkerhedszone til handlingsikonerne (`+`, `✓`, `🗑️`), så de aldrig dækkes eller blokeres af scrollbaren.

## 📋 Acceptance Criteria
- [x] **AC1 - 6px Ergonomisk Scrollbar Bredde**: `palette::SCROLLBAR_WIDTH` og `class_list::SCROLLBAR_WIDTH` øges fra 4.0px til 6.0px (med afrundet 3.0px radius), hvilket gør scrollbaren nem at ramme og gribe med musen.
- [x] **AC2 - Højrekantsflugt for Scrollable**: Det ydre panelcontainer-padding ændres fra ensartet `12px` til `[12, 2, 12, 12]`, så scrollable-elementet spænder helt ud til panelets højre kant med en 2px fuge mod hjørneradius.
- [x] **AC3 - Beskyttet Sikkerhedszone for Handlingsikoner**: Indre listekolonner (`concept_items`, `class_items`, `enum_items`) tildeles en indre højre padding (10-12px), så `✓`, `+` og `🗑️` kontrollerne bevarer deres eksisterende position med en fri korridor til scrollbaren uden risiko for blokering af klik eller hover.
- [x] **AC4 - Flugtende Header Layout**: `palette_header` og `enum_header` tilpasses med 12px højre padding, så søgefelt og knapper flugter harmonisk med listekortene nedenunder.
- [x] **AC5 - Ensartethed på tværs af Modeller**: Begrebsdiagram (Fane 2) og Informationsmodel (Fane 3) har identisk scrollbar-ergonomi, kantsikring og visuel flugt.

## 🚫 Must NOT
- Må IKKE fjerne eller forrykke `+`, `🗑️` eller `✓` handlingskontroller.
- Må IKKE formindske den effektive bredde til begrebs- eller klassenavne (244px indholdsbredde bevares).
- Må IKKE indføre permanent grå baggrundsskinne (rail forbliver transparent, auto-hide ved inaktivitet bevares).
- Må IKKE bryde drag-and-drop (`PaletteDragItem`) eller combobox-navigation (`Pil Op` / `Pil Ned` / `Enter`).

## 📝 Revisions
- 2026-10-27: Oprettet opgavepakke efter brugerfeedback om scrollbar-kantskift og 6px ergonomisk bredde.
- 2026-10-27: Gennemført og verificeret. 6px scrollbar, 2px kantfuge og 10px indre sikkerhedszone implementeret og forseglet.


## 🧪 Verifikation
- `xgauntlet check-spec -t 095`
- `cargo test --test palette_scrollbar_edge_alignment_acceptance`
- `cargo test --test palette_scrollbar_clearance_acceptance`
