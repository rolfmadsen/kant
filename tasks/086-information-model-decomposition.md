---
type: Task Package
title: "Task 086: Information Model Decomposition"
description: "Dekomponering af src/ui/information_model_view.rs (1.923 linjer) og src/features/information_model/mod.rs (1.719 linjer) i modulære feature-slices under 800 linjer"
status: active
generated: { by: process:antigravity-task-init, at: "2026-10-18T18:01:00Z" }
tags: [refactor, information-model, decomposition, modularity, architecture, ratchet]
scope:
  - "src/ui/information_model_view.rs"
  - "src/features/information_model/**"
  - "gauntlet.toml"
  - "tests/architecture_test.rs"
test_refs:
  - "tests/features/information_model_acceptance_test.rs"
  - "tests/architecture_test.rs"
---

# Task 086: Information Model Decomposition

**Status**: `ACTIVE`  
**Intent**: `🔄 REFACTOR`  
**Oprettet**: `2026-10-18`  

## 🎯 Formål
Dekomponere de to store monolitter i informationsmodel-domænet, så samtlige filer bringes under grænsen på 800 linjer jf. `Package-by-Feature` princippet:
1. Opdele `src/ui/information_model_view.rs` (1.923 linjer) i modulære UI-komponenter under `src/features/information_model/ui/`.
2. Dekomponere `src/features/information_model/mod.rs` (1.719 linjer) i separate domæne-moduler for datatyper, validering og begrebslineage.
3. Fjerne begge undtagelser fra `gauntlet.toml`, da alle berørte filer overholder grænsen på $\le 800$ linjer.

## 📋 Acceptance Criteria
- [ ] **AC1 (Domænemodel Dekomponering)**: Dekomponer `src/features/information_model/mod.rs` i `models.rs` ($\le 600$ linjer), `validation.rs` ($\le 500$ linjer) og `lineage.rs` ($\le 400$ linjer) med ren facade i `mod.rs`.
- [ ] **AC2 (UI Modul: Klasseliste & Metadata)**: Opret `src/features/information_model/ui/class_list.rs` ($\le 500$ linjer) og `class_detail.rs` ($\le 500$ linjer) til navigation og klasse-metadata.
- [ ] **AC3 (UI Modul: Attributter & Associationer)**: Opret `src/features/information_model/ui/attribute_editor.rs` ($\le 600$ linjer) med lineage-vælger samt `association_editor.rs` ($\le 500$ linjer) til UML-relationer.
- [ ] **AC4 (Fjernelse af Undtagelser i gauntlet.toml)**: Da alle dekomponerede filer er under 800 linjer, fjernes undtagelserne for `src/ui/information_model_view.rs` og `src/features/information_model/mod.rs` fra `gauntlet.toml`.
- [ ] **AC5 (Regression & Verifikation)**: Samtlige enheds- og accepttests i workspace passerer uden fejl, og `xgauntlet check-architecture` rapporterer 0 overtrædelser.

## 🚫 Must NOT
- Må IKKE ændre FDA informationsmodellens logik, valideringsregler eller serialiseringsformat.
- Må IKKE skabe nye filer over 800 linjer.
- Må IKKE lempes på nogen spærrehager.

## 📝 Revisions
- 2026-10-18: Oprettet opgavepakke til dekomponering af informationsmodel visning og domænelag.

## 🧪 Verifikation
- `cargo test --workspace`
- `cargo test --test architecture_test`
- `xgauntlet check-architecture`
- `xgauntlet verify`
- `xgauntlet check-spec -t 086-information-model-decomposition`
