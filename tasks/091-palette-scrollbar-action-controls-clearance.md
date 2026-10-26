---
type: Task Package
title: "Task 091: Palette Scrollbar Action Controls Clearance"
description: "Sikrer at den vertikale scrollbar i venstrepaletten på Begrebsdiagram og Informationsmodel ikke dækker over handlingsikonerne (+ og slet/skraldespand)."
status: done
generated: { by: process:antigravity-task-init, at: "2026-10-26T20:53:00Z" }
tags: [usability, ui, palette, scrollbar, layout, styling, concept-model, information-model]
scope:
  - "src/ui/theme.rs"
  - "src/features/concept_model/ui/palette.rs"
  - "src/features/information_model/ui/class_list.rs"
test_refs:
  - "tests/features/palette_scrollbar_clearance_acceptance_test.rs"
---

# Task 091: Palette Scrollbar Action Controls Clearance

**Status**: `DONE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-10-26`

## 🎯 Formål
Løse en visuel og funktionel usability-fejl i venstrepaletten på en elegant måde: I stedet for en klodset, permanent højre-gutter, implementeres en moderne svævende auto-hide scrollbar (4px slank afrundet pill, transparent rail uden grå baggrund, usynlig ved inaktivitet og fremhævet ved hover), så kort og handlingsknapper (`+`, `🗑️`, `✓`) bevarer en naturlig, afbalanceret højremargin.

## 📋 Acceptance Criteria
- [x] **AC1 - Elegant Auto-Hide Scrollbar Styling**: Scrollbaren i paletten er usynlig ved inaktivitet (`Status::Active` -> transparent scroller og transparent rail). Ved hover eller drag over paletten tones en diskret, afrundet pill frem.
- [x] **AC2 - Slank Bredde Uden Baggrundsskinne**: Scrollbarens bredde reduceres fra standard 10px til 4px (`Scrollbar::new().width(4.0).scroller_width(4.0)`), og der tegnes ingen uigennemsigtig grå baggrundsskinne (`Rail background: None`).
- [x] **AC3 - Fjernelse af Asymmetrisk Tom Margin**: Den kunstige 10px højre-padding og 4px tomme afstand fjernes fra elementerne, så `✓` og `🗑️` ikke efterlades med et tomt gab til højre for ikonet.
- [x] **AC4 - Ensartet Løsning på Tværs af Modeller**: Den æstetiske auto-hide scrollbar og balancerede layout anvendes identisk på Begrebsdiagram (Fane 2) og Informationsmodel (Fane 3).

## 🚫 Must NOT
- Må IKKE fjerne `+`, `🗑️` eller `✓` knapper/indikatorer fra paletten.
- Må IKKE bryde drag-and-drop (`PaletteDragItem`) eller combobox-navigation (`Pil Op` / `Pil Ned` / `Enter`).
- Må IKKE ændre eksisterende diagrammer eller modeller på canvas.

## 📝 Revisions
- 2026-10-26: Oprettet opgavepakke efter bruger-rapportering med skærmbillede af overlappet skraldespandsikon.
- 2026-10-26: Gennemført og verificeret med fast gutter.
- 2026-10-26: Genåbnet efter brugerfeedback på æstetikken. Skift fra klodset fast gutter til moderne svævende auto-hide scrollbar (4px pill, transparent rail, nul spildmargin).
- 2026-10-26: Gennemført og verificeret. Svævende auto-hide scrollbar (4px pill, transparent rail, afbalancerede kortmarginer) implementeret og forseglet.

## 🧪 Verifikation
- `xgauntlet check-spec -t 091`
- `cargo test --test palette_scrollbar_clearance_acceptance`
- `xgauntlet verify -t 091`
