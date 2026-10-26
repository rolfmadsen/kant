---
type: Task Package
title: "Task 091: Palette Scrollbar Action Controls Clearance"
description: "Sikrer at den vertikale scrollbar i venstrepaletten på Begrebsdiagram og Informationsmodel ikke dækker over handlingsikonerne (+ og slet/skraldespand)."
status: active
generated: { by: process:antigravity-task-init, at: "2026-10-26T20:53:00Z" }
tags: [usability, ui, palette, scrollbar, layout, concept-model, information-model]
scope:
  - "src/features/concept_model/ui/palette.rs"
  - "src/features/information_model/ui/class_list.rs"
test_refs:
  - "tests/features/palette_scrollbar_clearance_acceptance_test.rs"
---

# Task 091: Palette Scrollbar Action Controls Clearance

**Status**: `ACTIVE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-10-26`

## 🎯 Formål
Løse en visuel og funktionel usability-fejl i venstrepaletten, hvor den vertikale scrollbar dækker over handlingsknapperne (`+ Tilføj til diagram` og `🗑️ Slet`) samt `✓` statustegnet i både Fane 2 (Begrebsdiagram) og Fane 3 (Informationsmodel).

## 📋 Acceptance Criteria
- [ ] **AC1 - Højremargin for handlingskontroller i Begrebspaletten (Fane 2)**: Elementrækker i Begrebspaletten har tilstrækkelig højre-afstand/gutter til scrollbaren, så `+` og `🗑️` ikonerne aldrig overlappes af scrollbaren.
- [ ] **AC2 - Højremargin for handlingskontroller i Klasselisten (Fane 3)**: Klasserækker i Fane 3's venstrepalet har tilstrækkelig højre-afstand/gutter til scrollbaren, så `+` og `🗑️` ikonerne aldrig overlappes af scrollbaren.
- [ ] **AC3 - Højremargin for handlingskontroller i Enumerationslisten (Fane 3)**: Enumerationsrækker i Fane 3's venstrepalet har tilstrækkelig højre-afstand/gutter til scrollbaren, så `+` og `🗑️` ikonerne aldrig overlappes af scrollbaren.
- [ ] **AC4 - Udvidet Paletbredde for Læsbarhed**: Palettens bredde øges harmonisk fra 240px til 268px på tværs af Begrebsdiagram og Informationsmodel, hvilket giver plads til både scrollbar-gutteren og længere termer/navne.

## 🚫 Must NOT
- Må IKKE fjerne `+`, `🗑️` eller `✓` knapper/indikatorer fra paletten.
- Må IKKE bryde drag-and-drop (`PaletteDragItem`) eller combobox-navigation (`Pil Op` / `Pil Ned` / `Enter`).
- Må IKKE ændre eksisterende diagrammer eller modeller på canvas.

## 📝 Revisions
- 2026-10-26: Oprettet opgavepakke efter bruger-rapportering med skærmbillede af overlappet skraldespandsikon.

## 🧪 Verifikation
- `xgauntlet check-spec -t 091`
- `cargo test --test palette_scrollbar_clearance_acceptance`
- `xgauntlet verify -t 091`
