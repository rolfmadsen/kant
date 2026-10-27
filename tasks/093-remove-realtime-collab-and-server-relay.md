---
type: Task Package
title: "Task 093: Remove Realtime Collaboration and Server Relay"
description: "Fuldstændig sanering og fjernelse af realtids-kollaborering (src/features/collab), Axum relay-server (crates/kant-relay), ops-konfigurationer, krypto/netværks-dependencies samt Collab UI-elementer i Kant"
status: active
generated: { by: process:antigravity-task-init, at: "2026-10-27T19:54:00Z" }
tags: [refactor, cleanup, zero-daemon, architecture]
---

# Task 093: Remove Realtime Collaboration and Server Relay

**Status**: `ACTIVE`
**Intent**: `🔄 REFACTOR`
**Oprettet**: `2026-10-27`

## 🎯 Formål
Fuldstændig fjernelse af realtids-samarbejdsfunktionen ("Samarbejde") og den tilhørende WebSocket relay server (`crates/kant-relay`). Kant konsolideres omkring sin primære FDA-modelleringskerne og asynkront Git-baseret samarbejde via dekomponeret lagring (ADR 011).

## 📋 Acceptance Criteria
- [ ] **AC1 - Crate & Ops Sanering**: `crates/kant-relay/`, `koyeb.yaml`, `docker-compose.yml` og `.github/workflows/relay-docker.yml` er slettet, og `crates/kant-relay` er fjernet fra `Cargo.toml` (`[workspace.members]` og `[dev-dependencies]`).
- [ ] **AC2 - Cargo Dependency Rengøring**: `chacha20poly1305`, `tokio-tungstenite`, `base64`, `rand`, `bytes`, `futures-util` og `axum` er fjernet fra `Cargo.toml`.
- [ ] **AC3 - Feature Modul Sanering**: `src/features/collab/` er slettet, og `pub mod collab;` er fjernet fra `src/features/mod.rs`.
- [ ] **AC4 - UI & App State Forenkling**: `MenuType::Collab`, `menu_button("Samarbejde", ...)`, collab header badge, `CollabState`, `RelayServerPreset`, collab-felter i `App` og `Message::Collab*` beskeder er fjernet fra `src/ui/app.rs`.
- [ ] **AC5 - Verifikation & Testsuite Integritet**: `tests/features/collab_acceptance_test.rs` er slettet, [tests/acceptance.rs](tests/acceptance.rs) er renset for forældede collab-tests, og en ny accepttest `test_task093_collab_and_relay_purged` beviser at applikationen bygger og fungerer uden collab.

## 🚫 Must NOT
- Må IKKE efterlade ubrugte netværkssockets, baggrundstråde eller døde afhængigheder (Zero-Daemon invariant).
- Må IKKE introducere regressioner i FDA domænemodellen, lærredet, begrebslisten, informationsmodellen eller Git-integrationen.
- Må IKKE bryde linjelofterne specificeret i `gauntlet.toml`.

## 📝 Revisions
- 2026-10-27: Oprettet opgavepakke til fuldstændig udrensning af collab og kant-relay.

## 🧪 Verifikation
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy -- -D warnings`
- `xgauntlet check-spec`
