---
type: Task Package
title: "Task 088: Robust Excel Import and Diagnostic Logging"
description: "Løsning af fejl ved import af FDA Excel-skabelon (ufuldstændig arkudvælgelse, overskriftsdetektion der forveksler data med headers, og manglende hændelseslogning i diagnostik-panelet)"
status: active
generated: { by: process:antigravity-task-init, at: "2026-10-26T17:16:30Z" }
tags: [bugfix, excel, import, validation, fda, diagnostics, logging]
scope:
  - "src/features/concepts/excel/importer.rs"
  - "src/features/concepts/excel/mod.rs"
  - "src/ui/app.rs"
  - "tests/acceptance.rs"
test_refs:
  - "tests/acceptance.rs"
---

# Task 088: Robust Excel Import and Diagnostic Logging

**Status**: `ACTIVE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-10-26`

## 🎯 Formål
Sikre at et udfyldt Excel-dokument baseret på den officielle FDA Begrebslisteskabelon (`Begrebsliste_i_tabelformat_skabelon.xlsx`) altid indlæses og importeres fejlfrit, samt at importens trin og eventuelle afvisninger logges transparent til diagnostik-panelet (*Hjælp > System- og Grafikdiagnostik*).

## 📋 Acceptance Criteria
- [ ] **AC1 - Intelligent Ark-Udvælgelse (FDA Skabelon Kompatibilitet)**: Når en projektfil eller skabelon indlæses, evaluerer parseren de tilgængelige ark og vælger det ark, der reelt indeholder databegreber. Hvis `Begrebsliste DA` er udfyldt og `Begrebsliste DA+EN` er tom, skal `Begrebsliste DA` parses og importeres.
- [ ] **AC2 - Robust Kolonnedetektion Uden Data-Korruption**: Kolonnehoveder identificeres præcist. En gyldig etableret kolonne-mapping må IKKE overskrives eller korrumperes af efterfølgende datarrækker, selvom definitioner eller noter indeholder ord som "definition", "forklaring" eller "betydning".
- [ ] **AC3 - Eliminering af Falske Positiver i Header-Filter**: Gyldige begreber hvis foretrukne term starter med f.eks. "Foretrukken..." (f.eks. *Foretrukken leverandør*) eller hvis definition starter med "Definition..." må IKKE kastes bort som gentagne overskrifter.
- [ ] **AC4 - Struktureret Diagnostisk Logning**: Excel-operationer (filåbning, valgt ark, antal fundne begreber, antal afviste rækker med årsag, samt gennemført upsert) logges med målet `[excel]` til platformens in-memory og fil-logger via `crate::features::diagnostics`, så hændelserne fremgår i System- og Grafikdiagnostik.

## 🚫 Must NOT
- Må IKKE ændre eksisterende begrebers interne `Uuid` under upsert (det ville ødelægge diagram-noder og relationer).
- Må IKKE fejle i eksisterende tests for tosproget (DA+EN) import eller standard eksport.
- Må IKKE sluge fejl eller bruge usynlige `eprintln!` uden diagnostik-logning.

## 📝 Revisions
- 2026-10-26: Oprettet opgavepakke efter rodårsagsanalyse af fejlende skabelon-import og manglende diagnostik-logs.

## 🧪 Verifikation
- `cargo test --test acceptance test_task_088_robust_excel_import`
- `cargo test --test acceptance test_task_043`
- `cargo clippy --all-targets`
- `cargo fmt --check`
