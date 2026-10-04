---
type: Task Package
title: "Task 068: Nyt Kant Logo, Multiplatform Applikationsikoner & Palet-Ergonomi"
description: "Integration af nyt kant_logo.svg vektorlogo i headeren, opdatering af Linux/macOS/Windows app-ikoner (.rgba og .ico) samt flytning af palet-toggle til palet-header og canvas-toolbar jf. industristandard"
status: done
generated: { by: process:antigravity-task-init, at: "2026-10-04T11:33:00Z" }
tags: [ui, branding, logo, icons, desktop, linux, windows, macos, palette, ergonomics]
---

# Task 068: Nyt Kant Logo, Multiplatform Applikationsikoner & Palet-Ergonomi

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-10-04`

## 🎯 Formål
At opgradere Kant applikationens visuelle identitet med det nye officielle vektorlogo (`assets/kant_logo.svg`), sikre konsistent opdatering af alle OS-specifikke applikationsikoner (Linux SVG, Windows multi-resolution ICO, runtime 32x32 RGBA) og omplacere palet-toggle knappen (`Ctrl+B`) fra den globale header til palettens header og canvas-værktøjslinjen jf. etableret industristandard (Figma/VS Code).

## 📋 Acceptance Criteria
- [x] **AC1 - In-App Header Logo**: `assets/kant_logo.svg` er integreret og renderes som skarp vektorgrafik umiddelbart til venstre for teksten "Kant" i den øverste header-bar via `iced::widget::svg`.
- [x] **AC2 - Palet-Ergonomi (Industristandard)**: Den globale `sidebar_toggle_btn` før navnet "Kant" i headeren er fjernet.
  - Når venstrepaletten er synlig i Begrebsmodel og Informationsmodel, findes en kollaps-knap (`◨` / `◀`) i palettens egen header.
  - Når venstrepaletten er skjult, findes en "Vis palet (Ctrl+B)"-knap (`◨ Palet`) øverst til venstre på canvas-værktøjslinjen.
  - Global tastaturgenvej `Ctrl+B` bevarer fuld funktionalitet på tværs af visninger.
- [x] **AC3 - Multiplatform Applikationsikoner**:
  - `assets/icons/kant.svg` er opdateret til at matche `assets/kant_logo.svg`.
  - `assets/icons/kant-32.rgba` (præcis 32x32x4 = 4096 bytes) er regenereret ud fra det nye logo og verificeret af `load_window_icon()`.
  - `assets/icons/kant.ico` (multi-resolution ICO med 16x16, 32x32, 48x48, 256x256) er regenereret ud fra det nye logo til brug i `build.rs` / `winres`.
- [x] **AC4 - Headless & Verification Gauntlet**: Samtlige eksisterende og nye accepttests i `cargo test --workspace` består uden regressioner.

## 🚫 Must NOT
- Må IKKE fjerne eller forringe den globale tastaturgenvej `Ctrl+B` til at åbne/skjule venstrepaletten.
- Må IKKE bryde headless CI-kørsler (ingen display-server krav for enheds- eller accepttests).
- Må IKKE forringe opstartstid eller introducere runtime I/O under rendering af header-logoet (logo-bytes indlejres ved compile-time via `include_bytes!`).

## 📝 Revisions
- 2026-10-04: Oprettet opgavepakke efter brugergodkendelse.

## 🧪 Verifikation
- `cargo test --test acceptance test_task_068_`
- `cargo check --workspace`
- Visuel verifikation via subagent / browser hvis relevant.
