---
type: Task Package
title: "Task 083: Decouple Acceptance Tests & App Monolith"
description: "Dekomponering af monolitterne tests/acceptance.rs (12.181 linjer) og src/ui/app.rs (11.520 linjer) i isolerede feature-slices under faste linjegrænser"
status: active
generated: { by: process:antigravity-task-init, at: "2026-10-16T21:53:30Z" }
tags: [refactor, decomposition, modularity, architecture, ratchet]
scope:
  - "src/features/**"
  - "src/ui/**"
  - "tests/**"
  - "gauntlet.toml"
test_refs:
  - "tests/features/**"
---

# Task 083: Decouple Acceptance Tests & App Monolith

**Status**: `ACTIVE`  
**Intent**: `🔄 REFACTOR`  
**Oprettet**: `2026-10-16`  

## 🎯 Formål
Nedbryde de to gigantiske monolitter i Kant jf. Ratchet-arkitekturmodellen og `Package-by-Feature` princippet:
1. `tests/acceptance.rs` (12.181 linjer) dekomponeres i modulære feature-accepttests under `tests/features/` ($\le 1.000$ linjer hver).
2. `src/ui/app.rs` (11.520 linjer) dekomponeres ved at flytte feature-specifik view-, update- og event-håndtering ud i vertikale slices (`src/features/<feature>/ui/`) ($\le 800$ linjer hver).
3. Spærrehagen i `gauntlet.toml` strammes inkrementelt efter hver flytning, så koden aldrig kan vokse tilbage.

## 📋 Acceptance Criteria
- [ ] **AC1 (Dekomponering af Accepttests)**: Identificer mindst 3 sammenhængende test-domæner i `tests/acceptance.rs` og udskil dem til separate testfiler under `tests/features/<feature>_acceptance_test.rs` (hver under 1.000 linjer).
- [ ] **AC2 (Dekomponering af UI View & State)**: Udskil de tilsvarende view-komponenter og update-grene fra `src/ui/app.rs` til `src/features/<feature>/ui/mod.rs` (hver under 800 linjer).
- [ ] **AC3 (Ratchet Spærrehage Stramning)**: Reducer linjetallet i `tests/acceptance.rs` og `src/ui/app.rs` med mindst 20%, og opdater `max_lines` lofterne i `gauntlet.toml` tilsvarende.
- [ ] **AC4 (Ingen Adfærdsændringer / Regressioner)**: Samtlige eksisterende accepttests og enhedstests kører og består uden funktionelle regressionsfejl.
- [ ] **AC5 (Arkitektur & Gauntlet Verifikation)**: `xgauntlet check-architecture` og `xgauntlet verify` passerer 100%.

## 🚫 Must NOT
- Må IKKE ændre eksisterende forretningslogik eller brugeradfærd (assertions forbliver frosne under refaktorisering).
- Må IKKE oprette nye filer over 800 linjer eller testfiler over 1.000 linjer.
- Må IKKE hæve spærrehagen i `gauntlet.toml`.

## 📝 Revisions
- 2026-10-16: Task 083 oprettet til dekomponering af accepttests og UI-monolit i Kant.

## 🧪 Verifikation
- `cargo test --workspace`
- `xgauntlet check-architecture`
- `xgauntlet verify`
- `xgauntlet check-spec -t 083-decouple-acceptance-tests-and-app-monolith`
