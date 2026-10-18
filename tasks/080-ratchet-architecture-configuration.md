---
type: Task Package
title: "Task 080: Declarative Ratchet Architecture Configuration"
description: "Etablering af deklarativ [architecture] konfiguration i gauntlet.toml med forbud mod rå styling og ratchet spærrehage på app.rs og acceptance.rs"
status: active
generated: { by: process:antigravity-task-init, at: "2026-10-16T21:52:00Z" }
tags: [architecture, ratchet, gauntlet-toml, linter, limits, tokens]
scope:
  - "gauntlet.toml"
  - "tests/architecture_test.rs"
test_refs:
  - "tests/architecture_test.rs"
---

# Task 080: Declarative Ratchet Architecture Configuration

**Status**: `ACTIVE`  
**Intent**: `🚀 NEW FEATURE`  
**Oprettet**: `2026-10-16`  

## 🎯 Formål
Etablere xGauntlets deklarative arkitektur-gatekeeper i Kant, så arkitektonisk rot og uhæmmet vækst stoppes øjeblikkeligt:
1. Konfigurere de faste maksimale linjegrænser for produktion (`max_file_lines = 800`) og tests (`max_test_file_lines = 1000`).
2. Fastsætte absolutte spærrehager (`[[architecture.exemptions]]`) på samtlige 11 eksisterende filer der overskrider grænserne (herunder `src/ui/app.rs` på 11.550 linjer og `tests/acceptance.rs` på 12.200 linjer), så de fremadrettet KUN må falde i størrelse jf. Ratchet-modellen.
3. Forberede `forbidden_tokens = []` i `gauntlet.toml` til fremtidigt forbud mod rå ad-hoc styling (`Color::from_rgb`, `Color::from_rgba`), der aktiveres i Task 082 når det typesikre design system er introduceret.
4. Integrere arkitektur-kontrollen som et obligatorisk lag i Kants verifikations-gauntlet via `tests/architecture_test.rs` og `xgauntlet check-architecture`.

## 📋 Acceptance Criteria
- [x] **AC1 (Konfigurer [architecture] i gauntlet.toml)**: `gauntlet.toml` konfigureres med `max_file_lines = 800` og `max_test_file_lines = 1000`.
- [x] **AC2 (Forbered forbidden_tokens)**: `forbidden_tokens = []` deklareres i `gauntlet.toml`, forberedt til Task 082 forbud mod rå RGB farvetokens uden om design systemet.
- [x] **AC3 (Ratchet spærrehage på eksisterende monolitter)**: `[[architecture.exemptions]]` etableres for samtlige 11 eksisterende filer over grænseværdierne med faglige begrundelser og faste ratchet lofter.
- [x] **AC4 (Architecture Verification Layer)**: Laget `architecture` (`command = ["xgauntlet", "check-architecture"]`) tilføjes til `[[layers]]` i `gauntlet.toml` som non-optional.
- [x] **AC5 (Verifikation)**: `tests/architecture_test.rs` oprettes og verificerer konfigurationen samt at `xgauntlet check-architecture` passerer fejlfrit.

## 🚫 Must NOT
- Må IKKE hæve lofterne på `app.rs`, `acceptance.rs` eller øvrige undtagelser (spærrehagen må kun strammes, aldrig lempes).
- Må IKKE tillade nye filer over 800 linjer eller testfiler over 1.000 linjer.
- Må IKKE introducere eksterne netværks- eller dæmonafhængigheder (Zero-Daemon).

## 📝 Revisions
- 2026-10-16: Task 080 oprettet til udrulning af Ratchet-arkitektur i Kant.
- 2026-10-18: Afstemt ratchet-undtagelser for samtlige 11 legacy filer og forberedelse af forbidden_tokens til Task 082.

## 🧪 Verifikation
- `cargo test --test architecture_test`
- `xgauntlet check-architecture`
- `xgauntlet check-spec -t 080-ratchet-architecture-configuration`

