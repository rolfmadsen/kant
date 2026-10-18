---
type: Task Package
title: "Task 084: Full Design System Migration & Token Enforcement"
description: "Fuld udfasning af rå Color::from_rgb/rgba i produktionskode, harmonisering af theme.rs med ColorTokens og aktivering af forbidden_tokens i gauntlet.toml jf. ADR 015"
status: todo
generated: { by: process:antigravity-task-init, at: "2026-10-18T18:00:00Z" }
tags: [design-system, tokens, accessibility, wcag-aa, architecture, forbidden-tokens, theme]
scope:
  - "src/ui/theme.rs"
  - "src/ui/diagram_canvas.rs"
  - "src/ui/app.rs"
  - "src/ui/concept_editor.rs"
  - "src/ui/concept_model_view.rs"
  - "src/ui/concept_table.rs"
  - "src/ui/information_model_view.rs"
  - "src/features/git/ui/**"
  - "src/features/collab/ui/**"
  - "gauntlet.toml"
  - "tests/architecture_test.rs"
test_refs:
  - "tests/architecture_test.rs"
  - "tests/design_tokens_contrast_test.rs"
---

# Task 084: Full Design System Migration & Token Enforcement

**Status**: `TODO`  
**Intent**: `🔄 REFACTOR`  
**Oprettet**: `2026-10-18`  

## 🎯 Formål
Fuldende udrulningen af Kants tilgængelige design-system jf. ADR 015 og stoppe enhver brug af rå farver:
1. Harmonisere `src/ui/theme.rs` med `src/design_system/tokens.rs`, så alle tema-konstanter delegerer direkte til `ColorTokens`.
2. Migrere samtlige resterende ~60 rå `Color::from_rgb` og `Color::from_rgba` instanser i `src/ui/` og `src/features/*/ui/` til semantiske design tokens.
3. Aktivere `forbidden_tokens = ["Color::from_rgb", "Color::from_rgba"]` i `gauntlet.toml`, så fremtidig brug af rå farver fysisk blokeres af gatekeeperen.

## 📋 Acceptance Criteria
- [ ] **AC1 (ThemeColors Harmonering)**: `src/ui/theme.rs` omskrives så alle konstanter (`PRIMARY`, `SLATE_*`, `SURFACE_*`, `FDA_*` mv.) viderestiller direkte til `ColorTokens` fra `src/design_system/tokens.rs`, hvilket eliminerer 105 rå RGB-instanser.
- [ ] **AC2 (Resterende UI Migrering)**: Samtlige resterende rå `Color::from_rgb` og `Color::from_rgba` i `diagram_canvas.rs`, `app.rs`, `concept_editor.rs`, `concept_table.rs`, `concept_model_view.rs`, `information_model_view.rs` samt `features/*/ui/` erstattes med semantiske `ColorTokens`.
- [ ] **AC3 (Forbidden Tokens Lås i gauntlet.toml)**: `forbidden_tokens = ["Color::from_rgb", "Color::from_rgba"]` aktiveres i `[architecture]` sektionen i `gauntlet.toml`.
- [ ] **AC4 (Arkitektur-test Opdatering)**: `tests/architecture_test.rs` opdateres til at verificere, at `gauntlet.toml` deklarerer de forbudte tokens, og at `xgauntlet check-architecture` passerer med 0 overtrædelser.
- [ ] **AC5 (Verifikation & Nul Regressioner)**: `xgauntlet check-architecture`, `cargo test --workspace` og `xgauntlet verify` passerer 100% uden visuelle eller funktionelle regressioner.

## 🚫 Must NOT
- Må IKKE efterlade en eneste forekomst af `Color::from_rgb` eller `Color::from_rgba` i produktionskode (`src/`).
- Må IKKE kompromittere WCAG AA kontrastgarantierne ($\ge 4.5:1$) defineret i ADR 015 og verificeret i `design_tokens_contrast_test.rs`.
- Må IKKE overskride de etablerede linjelofter for nogen eksisterende filer i `gauntlet.toml`.

## 📝 Revisions
- 2026-10-18: Oprettet opgavepakke til fuld token-migrering og aktivering af forbidden_tokens gatekeeper.

## 🧪 Verifikation
- `cargo test --test design_tokens_contrast_test`
- `cargo test --test architecture_test`
- `xgauntlet check-architecture`
- `xgauntlet verify`
- `xgauntlet check-spec -t 084-full-design-system-migration-and-token-enforcement`
