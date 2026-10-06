---
type: Task Package
title: "Task 076: Asynkron Git Synkroniseringsstatus og Komplet Diagnostik Log-udklipsholder"
description: "Eliminering af UI-frys ved at flytte GitService::get_sync_status fra Iced UI-tråden til en koalesceret baggrundsopgave, samt inkludering af fuld hændelseslog i diagnostik-udklipsholderen."
status: done
generated: { by: process:antigravity-task-init, at: "2026-10-06T19:35:00Z" }
tags: [git, async, performance, diagnostics, clipboard, autosave, windows]
---

# Task 076: Asynkron Git Synkroniseringsstatus og Komplet Diagnostik Log-udklipsholder

**Status**: `DONE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-10-06`
**Scope**: `src/ui/app.rs`, `src/main.rs`, `src/features/diagnostics.rs`, `tests/acceptance.rs`
**Blokerer**: Task 077 (077 forudsætter at frysningen er væk, før dobbeltklik-adfærd kan vurderes)

## 🔬 Evidens (brugerrapport, Windows, v0.8.0)
```text
[+10240ms] [canvas]   Drag fuldført for 2 noder (distance: 18.1px, varighed: 864.5864ms)
[+12360ms] [autosave] Autosave gennemført (disk: 17.6866ms, git: 2.1015478s)   <- UI-tråd blokeret 2,1 s
[+12397ms] [autosave] Autosave gennemført (disk: 20.3722ms, git: 0ns)
```
- `App::new_with_path` starter med `RepoSyncStatus::Uninitialized` (Task 054). Første `trigger_autosave()` kalder derfor synkront `GitService::get_sync_status` (op til 5 `git.exe`-spawns) på UI-tråden.
- Efterfølgende autosaves viser `git: 0ns`, fordi status nu er `PendingChanges` og tjekket springes over.
- Efter en udgivelse sættes status til `Synced` → **næste autosave fryser igen**.
- Task 067 flyttede autosave fra `CursorMoved` til drop, men flyttede dermed blot frysningen til drop-øjeblikket.
- Musehændelser, der hober sig op under frysningen, afvikles i et burst bagefter. `DiagramCanvas` bruger `Instant::now()` ved *behandling*, så ophobede klik tolkes som dobbeltklik (→ utilsigtet `CanvasDoubleClicked`). Denne task fjerner årsagen; Task 077 håndterer canvas-siden.

## 🎯 Formål
1. Ingen kald til `GitService::get_sync_status` på Iced UI-tråden. Status hentes i en koalesceret baggrundsopgave og anvendes via en besked.
2. `git_sync_status` afspejler altid repositoryets faktiske tilstand — ingen optimistisk gætning, der kan lyve (ADR 011: deterministisk serialisering betyder, at en save uden reel ændring ikke giver git-diff).
3. "📋 Kopier rapport til udklipsholder" medtager hele in-memory hændelsesloggen.

## 🗺️ Call-site inventar (`GitService::get_sync_status`)
| Linje (v0.8.0) | Kontekst | Tråd i dag | Handling i denne task |
|---|---|---|---|
| 2738 | `trigger_autosave()` | UI | **Erstattes** af `request_git_status_refresh()` |
| 3031 | `Message::RefreshGitStatus` | UI | **Erstattes** |
| 4220 | Fil åbnet (`FileOpened`-flow) | UI | **Erstattes** |
| 3244 | `InitGitRepository` (status-delen) | UI | **Erstattes** (selve `init_repository` → se Uden for scope) |
| 3274 | `PullModel` (status-delen) | UI | **Erstattes** (selve `pull_model` → se Uden for scope) |
| 3383 | Konfliktløsning (status-delen) | UI | **Erstattes** (selve `publish_model`/`push_model` → se Uden for scope) |
| 3156, 3175 | `execute_publish_workflow` | `spawn_blocking` | Uændret (allerede off-thread) |
| — | Opstart (`new_with_path` / `new_auto_open`) | ingen | **Nyt**: initialt baggrundstjek ved boot, så status ikke står som `Uninitialized` indtil første autosave |

## 🏗️ Design (bindende)
1. **Tilstandsmaskine** på `App` (ikke en løs `bool`):
   ```rust
   enum GitStatusRefresh { Idle, InFlight { generation: u64 }, InFlightDirty { generation: u64 } }
   git_status_generation: u64   // bumpes ved FileOpened / ny model / Save As / repo-skift
   ```
   - `request_git_status_refresh()`: `Idle` → `InFlight` + returnér task; `InFlight` → `InFlightDirty` (ingen ny task); `InFlightDirty` → uændret.
   - Ingen repo (`repo_dir() == None`) → sæt `Uninitialized` direkte, ingen task.
2. **Ny besked**: `Message::GitSyncStatusRefreshed { generation: u64, result: Result<RepoSyncStatus, String>, elapsed: Duration }`.
   - `generation != git_status_generation` → resultatet kasseres (forældet svar fra tidligere model/repo) og maskinen går til `Idle`/genstart efter gældende regel.
   - `Ok(status)` → `git_sync_status = status`. `Err` → status uændret + `log_warn("git", ...)`.
   - `InFlightDirty` → start straks nyt tjek (et svar, der startede før seneste save, kan være forældet).
3. **Afvikling**: Samme mønster som `ConfirmPublish` (app.rs ~3196): `Task::perform(async { tokio::task::spawn_blocking(...).await }, ...)` når `tokio::runtime::Handle::try_current().is_ok()`; ellers synkron fallback (headless tests).
4. **Seam for `trigger_autosave()`** (≈70 kaldere, returnerer `()`): `trigger_autosave()` sætter kun `git_status_refresh_requested = true`. Nuværende `update()` omdøbes til `update_inner()`; ny `update()` kalder `update_inner()` og batcher `request_git_status_refresh()`-tasken, hvis flaget er sat. Ingen ændring af de ≈70 kaldsteder.
5. **Boot**: `app_creator` i `src/main.rs` returnerer `(App, Task<Message>)`, hvor tasken er det initiale statustjek (iced `application` accepterer en boot-funktion, der returnerer tuple).
6. **Telemetri**:
   - `Autosave gennemført (disk: {:?})` — git-målingen fjernes fra denne linje.
   - Ved resultat: `log_info("git", "Git-status opdateret: {status:?} (varighed: {elapsed:?}, generation: {g})")`; `log_warn` hvis `elapsed > 500ms`; `log_info` når et forældet svar kasseres.
7. **Diagnostik-rapport**:
   - `SystemDiagnostics` får feltet `recent_logs: Vec<String>`, udfyldt i `get_system_diagnostics()` fra `get_recent_logs()` (kronologisk, op til `MAX_LOG_ENTRIES = 1000`).
   - `formatted_report()` tilføjer efter milepælene:
     ```text
     -----------------------------------------------------
       Seneste loghændelser (In-memory buffer, N linjer):
     [+0ms] [INFO] [startup] ...
     ...
     ```
   - Tom buffer → `  (Ingen logbeskeder registreret endnu)`.
   - `--diagnostics` CLI (`src/main.rs`) får samme output (accepteret konsekvens).
   - Dialogens visning (30 nyeste, omvendt rækkefølge) er uændret.

## 📋 Acceptance Criteria
- [x] **AC1 – Ingen synkron statuskald i autosave**: `trigger_autosave()` ændrer ikke `git_sync_status` og kalder ikke `GitService`; den sætter kun refresh-flaget. Bevis: test i et midlertidigt git-repo, hvor status før og umiddelbart efter `trigger_autosave()` er uændret, og `git_status_refresh_state()` er `InFlight` efter `update()`.
- [x] **AC2 – Koalescering**: N på hinanden følgende autosaves mens et tjek kører giver højst ét efterfølgende tjek (`InFlight` → `InFlightDirty` → ét nyt tjek ved svar). Testes direkte på tilstandsmaskinen via `Message::GitSyncStatusRefreshed`.
- [x] **AC3 – Forældede svar kasseres**: Et `GitSyncStatusRefreshed` med gammel `generation` (fx efter `FileOpened` af anden model) ændrer ikke `git_sync_status`.
- [x] **AC4 – Sandfærdig status**: I et midlertidigt git-repo med commit: save uden reel modelændring → `Synced`; save med ændring → `PendingChanges`. `UnpublishedCommits(n)` overskrives ikke af autosave alene.
- [x] **AC5 – Alle UI-tråds call-sites erstattet**: Linjerne 2738, 3031, 3244, 3274, 3383 og 4220 bruger `request_git_status_refresh()`. Invariant-test: `include_str!("../src/ui/app.rs")` indeholder `get_sync_status(` kun i baggrundshelperen og i `execute_publish_workflow`.
- [x] **AC6 – Initialt tjek ved boot**: Når en model i et git-repo åbnes ved opstart, returnerer boot en task, der opdaterer status uden at blokere første frame.
- [x] **AC7 – Fuld log i udklipsholderen**: `formatted_report()` indeholder sektionen `Seneste loghændelser`, alle buffer-linjer i kronologisk rækkefølge (første før sidste), og eksisterende rapport-tests (acceptance.rs ~8660 og ~8760) passerer fortsat.
- [x] **AC8 – Regression Gauntlet**: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check` og `xgauntlet verify` er grønne.

## 🚫 Must NOT
- Må IKKE kalde `GitService::get_sync_status` på UI-tråden (håndhæves af AC5-invarianttesten).
- Må IKKE gætte status optimistisk (fx sætte `PendingChanges` direkte efter save).
- Må IKKE starte mere end ét samtidigt statustjek.
- Må IKKE efterlade baggrundstråde efter appens afslutning ud over tokio's egen blocking-pool (Zero-Daemon).
- Må IKKE ændre `.kant`-format eller serialisering.
- Zero Ambient Authority: ingen netværkstelemetri. Statustjek må ikke kalde `git fetch` eller anden netværkstrafik.
- Må IKKE udføre `git push` eller destruktive git-handlinger.

## ⛔ Uden for scope (opfølgning anbefales som Task 078)
Følgende kalder stadig **netværks- eller tunge git-operationer synkront på UI-tråden** og kan fryse længere end 2 s. De rettes ikke her, men er registreret, så "Zero-Freeze" ikke påstås opfyldt globalt:
- `Message::PullModel` → `GitService::pull_model` (~3256)
- Konfliktløsning → `GitService::publish_model` + `push_model` (~3377–3382)
- `Message::InitGitRepository` → `GitService::init_repository` (~3243)
- `Message::OpenModelHistoryModal` → `GitService::get_commit_history` (~3281)
- `Message::OpenGitConnectionModal` → `GitService::get_remote_url` m.fl. (~3394)
- `diagnostics::log_entry` åbner logfilen synkront ved hver linje (lav prioritet).

## 🧪 RED-hypoteser
1. `test_task_076_autosave_does_not_touch_git_status_synchronously` fejler i dag, fordi `trigger_autosave()` sætter `git_sync_status` fra `Uninitialized` til en reel status i samme kald.
2. `test_task_076_stale_git_status_result_is_discarded` fejler i dag, fordi beskeden/tilstandsmaskinen ikke findes (kompileringsfejl tæller som RED).
3. `test_task_076_report_contains_full_log_buffer` fejler i dag, fordi `formatted_report()` ikke indeholder `Seneste loghændelser`.
4. `test_task_076_no_ui_thread_get_sync_status_calls` fejler i dag på 6 forekomster i `app.rs`.

## 📝 Revisions
- 2026-10-06: Oprettet opgavepakke efter telemetri-analyse af 2,1 s UI-frys forårsaget af synkrone `GitService::get_sync_status`-kald under autosave på Windows samt brugeranmodning om log i udklipsholder.
- 2026-10-06: Skærpet efter review: call-site inventar, eksplicit scope-grænse (netværks-git → Task 078), tilstandsmaskine med generation og dirty-flag, `update()`-seam, initialt boot-tjek, testbare AC'er og RED-hypoteser.

## 🧪 Verifikation
- `cargo test --test acceptance test_task_076_`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
- `xgauntlet check-spec -t 076`
- Manuel Windows-test: åbn model i git-repo, træk 2 noder straks efter opstart → ingen frys; loggen viser `Git-status opdateret` med varighed.
