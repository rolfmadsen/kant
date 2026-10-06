---
type: Task Package
title: "Task 076: Asynkron Git Synkroniseringsstatus og Komplet Diagnostik Log-udklipsholder"
description: "Eliminering af UI-frys på Windows ved omlægning af GitService statuskald til asynkron baggrundsafvikling samt inkludering af fuld hændelseslog i diagnostik-udklipsholderen."
status: active
generated: { by: process:antigravity-task-init, at: "2026-10-06T19:35:00Z" }
tags: [git, async, performance, diagnostics, clipboard, autosave, windows]
---

# Task 076: Asynkron Git Synkroniseringsstatus og Komplet Diagnostik Log-udklipsholder

**Status**: `ACTIVE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-10-06`
**Scope**: `src/ui/app.rs`, `src/features/diagnostics.rs`, `tests/acceptance.rs`

## 🎯 Formål
Løse det observerede 2,1 sekunders UI-frys på Windows samt forbedre diagnostik-rapporteringen:
1. **Asynkron Git-synkroniseringsstatus**:
   - Fjerne enhver synkron eksekvering af `GitService::get_sync_status` på Iced UI-hovedtråden i `trigger_autosave()` (linje 2738), `Message::FileOpened` (linje 4220), `Message::RefreshGitStatus` (linje 3031) og øvrige reaktive steder.
   - Implementere et asynkront mønster: `tokio::task::spawn_blocking` + `Task::perform(..., Message::GitSyncStatusUpdated)` i overensstemmelse med den etablerede async publish arkitektur.
   - Etablere en `git_sync_in_progress: bool` lås/guard så gentagne autosaves ikke affyrer parallelle Git-subprocesser, men debounces og afvikles i baggrunden uden UI-lag.
   - Sikre at `trigger_autosave()` afvikles på 0 ms hvad angår Git, og at `git_sync_status` altid afspejler sandfærdig repository-tilstand uden at overskrive `UnpublishedCommits`, `Synced` eller maskere deterministiske no-op saves (ADR 011).
2. **Komplet Diagnostik-udklipsholder**:
   - Udvide `formatted_report()` i `src/features/diagnostics.rs` til at medtage samtlige loghændelser fra in-memory bufferen (`get_recent_logs()`, op til `MAX_LOG_ENTRIES = 1000`) i kronologisk rækkefølge.
   - Et enkelt klik på knappen "📋 Kopier rapport til udklipsholder" kopierer dermed hele hændelsesforløbet fra processtart inklusive opstartsmilepæle, autosave-målinger, canvas-aktiviteter og fejl.

## 📋 Acceptance Criteria
- [ ] **AC1 - Ikke-blokerende Autosave Git-status**: `trigger_autosave()` udfører aldrig synkrone Git proceskald på UI-hovedtråden. Hvis modellen er i et Git-repository, initieres et asynkront baggrundstjek uden at blokere frame-budgettet.
- [ ] **AC2 - In-Flight Beskyttelse & Debouncing**: Der startes højst ét asynkront Git-statustjek ad gangen (`git_sync_in_progress`). Hvis en ny gemning sker mens et tjek kører, markeres behov for gen-tjek, som afvikles når det igangværende fuldføres.
- [ ] **AC3 - Præcis Git Status-bevarelse**: Asynkront Git-tjek bevarer korrekte tilstande (`RepoSyncStatus::Synced`, `PendingChanges`, `UnpublishedCommits(N)`), og no-op saves (hvor diskens model er uændret jf. ADR 011 deterministisk serialisering) overskriver ikke status forkert.
- [ ] **AC4 - Ikke-blokerende Filåbning og Manuel Opdatering**: `Message::FileOpened` og `Message::RefreshGitStatus` benytter samme asynkrone task-flow uden UI-stutter.
- [ ] **AC5 - Komplet Diagnostik Hændelseslog i Udklipsholder**: `formatted_report()` inkluderer hele in-memory logbufferen (kronologisk, op til 1.000 linjer) under en veldefineret sektion `Seneste loghændelser (In-memory buffer):`.
- [ ] **AC6 - Regression Gauntlet**: Alle workspace enhedstests, accepttests og clippy warnings forbliver 100% grønne.

## 🚫 Must NOT
- Zero-Freeze invariant: Må ALDRIG kalde synkrone subprocesser på UI-hovedtråden.
- Zero-Daemon invariant: Må IKKE efterlade hængende baggrundstråde eller uovervågede tokio tasks.
- Zero Ambient Authority: Ingen netværkstelemetri eller eksterne kald.
- Må IKKE ændre eksisterende diagrammer eller `.kant` serialiseringsformat.
- Må IKKE udføre remote push eller destruktive git handlinger (`git reset --hard` / `git clean -f`).

## 📝 Revisions
- 2026-10-06: Oprettet opgavepakke efter telemetri-analyse af 2,1 sekunders UI-frys forårsaget af synkrone `GitService::get_sync_status`-kald under autosave på Windows samt brugeranmodning om log i udklipsholder.

## 🧪 Verifikation
- `cargo test --test acceptance test_task_076_`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets`
- `cargo fmt --check`
