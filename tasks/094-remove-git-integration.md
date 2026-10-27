---
type: Task Package
title: "Task 094: Remove Git Integration"
description: "Fuldstændig sanering og fjernelse af indbygget Git-integration (src/features/git), 3-vejs merge motor (src/features/model/merge.rs) samt tilhørende Git UI-elementer, modaler og tests i Kant"
status: active
generated: { by: process:antigravity-task-init, at: "2026-10-27T21:17:00Z" }
tags: [refactor, cleanup, zero-daemon, architecture, git-purge]
scope:
  - "src/features/git/**"
  - "src/features/model/merge.rs"
  - "src/ui/app.rs"
  - "Cargo.toml"
  - "tests/**"
test_refs:
  - "tests/features/git_purge_acceptance_test.rs"
---

# Task 094: Remove Git Integration

**Status**: `ACTIVE`
**Intent**: `🔄 REFACTOR`
**Oprettet**: `2026-10-27`

## 🎯 Formål
Fuldstændig fjernelse af den indbyggede Git-integration (`src/features/git`) og den dertilhørende semantiske 3-vejs merge motor (`src/features/model/merge.rs`). Kant rendyrkes som en fokuseret FDA-modelleringsapplikation med lokal diskpersistens (herunder `.kant/` dekomponeret katalog-lagring), mens versionsstyring overlades til brugerens eksterne værktøjer.

## 📋 Acceptance Criteria
- [ ] **AC1 - Modul Sanering**: `src/features/git/` er fuldstændig slettet, og `pub mod git;` er fjernet fra `src/features/mod.rs`.
- [ ] **AC2 - Merge Motor Sanering**: `src/features/model/merge.rs` er slettet, og `pub mod merge;` er fjernet fra `src/features/model/mod.rs`.
- [ ] **AC3 - UI & App State Forenkling**: Git status badge i headeren, Git-modaler (Publish, History, Clone, Connection, Conflict Resolver) og samtlige Git-relaterede beskeder og tilstandsfelter i `src/ui/app.rs` er fjernet uden efterladte referencer.
- [ ] **AC4 - Testsuite & Cargo Sanering**: `tests/features/git_acceptance_test.rs` er slettet, `Cargo.toml` er renset for `git_acceptance`, `tests/acceptance.rs` er renset for Git-tests, og `tests/architecture_test.rs` er opdateret.
- [ ] **AC5 - Dokumentation & Verifikation**: `docs/adr/011-decomposed-model-persistence-and-git-integration.md` og `spec.md` er opdateret, og en ny accepttest `tests/features/git_purge_acceptance_test.rs` beviser at applikationen bygger og fungerer fejlfrit uden Git.

## 🚫 Must NOT
- Må IKKE berøre eller fjerne den dekomponerede katalog-lagring (`.kant/` layout) i `src/features/persistence/`.
- Må IKKE efterlade ubrugte subprocess-kald, baggrundstråde eller døde beskeder i `App` (Zero-Daemon invariant).
- Må IKKE introducere regressioner i begrebsmodellen, informationsmodellen, lærredet, validering eller eksport.
- Må IKKE bryde linjelofterne specificeret i `gauntlet.toml`.

## 📝 Revisions
- 2026-10-27: Oprettet opgavepakke til fuldstændig sanering af Git-integration.

## 🧪 Verifikation
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy -- -D warnings`
