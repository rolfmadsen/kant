---
type: Task Package
title: "Task 088: Robust Excel Import and Diagnostic Logging"
description: "Løsning af fejl ved import af FDA Excel-skabelon (intelligent arkudvælgelse, fanevælger, udtrækning af forretningsmetadata, overskriftsdetektion der forveksler data med headers, og diagnostisk logning)"
status: done
generated: { by: process:antigravity-task-init, at: "2026-10-26T17:16:30Z" }
tags: [bugfix, excel, import, validation, fda, metadata, diagnostics, logging]
scope:
  - "src/features/concepts/excel/**"
  - "src/features/export/ui/**"
  - "src/ui/app.rs"
  - "tests/features/export_acceptance_test.rs"
test_refs:
  - "tests/features/export_acceptance_test.rs"
---

# Task 088: Robust Excel Import and Diagnostic Logging

**Status**: `DONE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-10-26`

## 🎯 Formål
Sikre at et udfyldt Excel-dokument baseret på den officielle FDA Begrebslisteskabelon (`Begrebsliste_i_tabelformat_skabelon.xlsx`) altid indlæses og importeres fejlfrit, give brugeren en fanevælger hvis flere ark har data, udtrække FDA Forretningsmetadata, samt logge alle importtrin og afvisninger transparent til diagnostik-panelet (*Hjælp > System- og Grafikdiagnostik*).

## 📋 Acceptance Criteria
- [x] **AC1 - Intelligent Ark-Udvælgelse & Fanevælger**: Når en projektfil eller skabelon indlæses, evaluerer parseren de tilgængelige ark og vælger automatisk det ark, der reelt indeholder databegreber (`Begrebsliste DA` frem for tomt `Begrebsliste DA+EN`). Hvis begge ark indeholder data, præsenteres brugeren for en fanevælger i forhåndsvisningsdialogen.
- [x] **AC2 - Udtrækning og Valgfri Opdatering af Forretningsmetadata**: Officielle FDA Forretningsmetadata udtrækkes fra arket `Forretningsmetadata`, hvis det findes, og brugeren kan via en afkrydsningsboks vælge også at opdatere modellens omslag/metadata.
- [x] **AC3 - Robust Kolonnedetektion Uden Data-Korruption**: Kolonnehoveder identificeres præcist. En gyldig etableret kolonne-mapping må IKKE overskrives eller korrumperes af efterfølgende datarrækker, selvom definitioner eller noter indeholder ord som "definition", "forklaring" eller "betydning".
- [x] **AC4 - Eliminering af Falske Positiver i Header-Filter**: Gyldige begreber hvis foretrukne term starter med f.eks. "Foretrukken..." (f.eks. *Foretrukken leverandør*) eller hvis definition starter med "Definition..." må IKKE kastes bort som gentagne overskrifter.
- [x] **AC5 - Struktureret Diagnostisk Logning**: Excel-operationer (filåbning, valgt ark, antal fundne begreber, antal afviste rækker med årsag, samt gennemført upsert) logges med målet `[excel]` til platformens in-memory og fil-logger via `crate::features::diagnostics`, så hændelserne fremgår i System- og Grafikdiagnostik.

## 🚫 Must NOT
- Må IKKE ændre eksisterende begrebers interne `Uuid` under upsert (det ville ødelægge diagram-noder og relationer).
- Må IKKE fejle i eksisterende tests for tosproget (DA+EN) import eller standard eksport.
- Må IKKE sluge fejl eller bruge usynlige `eprintln!` uden diagnostik-logning.

## 📝 Revisions
- 2026-10-26: Oprettet opgavepakke efter rodårsagsanalyse af fejlende skabelon-import og manglende diagnostik-logs.
- 2026-10-26: Implementeret robust arkudvælgelse, kolonnedetektion, fanevælger ved flere datark, Forretningsmetadata-udtrækning og diagnostiklogning.

## 🧪 Verifikation
- `cargo test --test export_acceptance test_task_088_robust_excel_import`
- `cargo test --test export_acceptance`
- `cargo test --test acceptance test_task_043`
- `cargo clippy --all-targets`
- `cargo fmt --check`
- `xgauntlet verify`

