---
type: Task Package
title: "Task 082: Typed Design System Tokens Implementation"
description: "Etablering af src/design_system/tokens.rs i Kant med type-sikre farver, typografi og afstande samt matematiske beviser for WAI 2.0 / WCAG AA >= 4.5:1 kontrast"
status: todo
generated: { by: process:antigravity-task-init, at: "2026-10-16T21:53:00Z" }
tags: [design-tokens, ui, iced, accessibility, wcag-aa, math-proof, tokens]
scope:
  - "src/design_system/**"
  - "src/lib.rs"
  - "src/main.rs"
test_refs:
  - "tests/design_tokens_contrast_test.rs"
---

# Task 082: Typed Design System Tokens Implementation

**Status**: `TODO`  
**Intent**: `🚀 NEW FEATURE`  
**Oprettet**: `2026-10-16`  

## 🎯 Formål
Etablere et centralt, type-sikkert design-tokens system i Kant under `src/design_system/tokens.rs`:
1. Definere semantiske farver, typografi og spacing-skalaer som førsteklasses typer til brug i Iced UI.
2. Tilføje enhedstests, der matematisk beregner relativ luminans jf. W3C specifikationen og beviser, at alle tekst/baggrundskombinationer overholder WCAG AA kravet ($\ge 4.5:1$).
3. Erstatte de første rå `Color::from_rgb` instanser i en udvalgt UI-komponent med de nye semantiske tokens.

## 📋 Acceptance Criteria
- [ ] **AC1 (Etablering af Design System Modul)**: Opret `src/design_system/tokens.rs` med typed tokens for farver (`ColorTokens`), typografi (`TypographyTokens`) og afstande (`SpacingTokens`).
- [ ] **AC2 (WCAG AA Matematisk Kontrast-beregning)**: Implementer funktion til beregning af relativ luminans og kontrastforhold jf. WCAG 2.1 formlen: `(L1 + 0.05) / (L2 + 0.05)`.
- [ ] **AC3 (Kontrast Acceptance Tests)**: Opret `tests/design_tokens_contrast_test.rs`, der asserter, at:
  - `TextPrimary` mod `Background` $\ge 4.5:1$
  - `TextPrimary` mod `Surface` $\ge 4.5:1$
  - `TextMuted` mod `Background` $\ge 4.5:1$
  - `Danger` / `Success` badges mod deres respektive baggrunde overholder kontrastkrav.
- [ ] **AC4 (Iced Kompatibilitet)**: Design tokens konverterer problemfrit til `iced::Color` uden runtime overhead.
- [ ] **AC5 (Pilot Migrering)**: Mindst én eksisterende UI-komponent migreres fra rå RGB til de nye tokens for at validere ergonomi.

## 🚫 Must NOT
- Må IKKE indeholde rå RGB-farveværdier, der ikke er underlagt kontrast-validering.
- Må IKKE overskride grænsen på 800 linjer for `src/design_system/tokens.rs`.
- Må IKKE bryde eksisterende kompilering eller tests i Kant.

## 📝 Revisions
- 2026-10-16: Task 082 oprettet til implementering af tilgængelige design tokens i Kant.

## 🧪 Verifikation
- `cargo test --test design_tokens_contrast_test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `xgauntlet check-spec -t 082-design-system-tokens-implementation`
