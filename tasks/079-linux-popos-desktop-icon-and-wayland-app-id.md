---
type: Task Package
title: "Task 079: Linux & Pop!_OS Desktop Ikon Integration og Wayland App ID"
description: "Eksplicit Wayland application_id i Iced, generering af faste PNG-ikoner og pixmap-fallback samt zero-privilege lokal desktop-integration for Pop!_OS COSMIC og Linux"
status: done
generated: { by: process:antigravity-task-init, at: "2026-10-15T16:05:00Z" }
tags: [desktop, linux, popos, cosmic, wayland, iced, icons, packaging, deb]
---

# Task 079: Linux & Pop!_OS Desktop Ikon Integration og Wayland App ID

**Status**: `DONE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-10-15`

## 🎯 Formål
At sikre at Kant automatisk vises med sit korrekte, officielle applikationsikon i docken (COSMIC Panel), applikationsvælgeren (COSMIC App Library) og Alt+Tab-skifteren på Pop!_OS og Linux under Wayland og X11. Dette eliminerer fallback-tandhjulet uden at kræve manuel opsætning fra brugerens side.

## 📋 Acceptance Criteria
- [x] **AC1 - Wayland & X11 Window Identity**: `default_window_settings()` i `src/ui/platform.rs` sætter eksplicit `settings.platform_specific.application_id = "kant".to_string()`, så Wayland compositors (Mutter, COSMIC `cosmic-comp`, KWin) og X11 (`WM_CLASS`) forbinder det kørende vindue med `kant.desktop`.
- [x] **AC2 - Multi-Resolution PNG Ikoner**:
  - `assets/icons/kant-48.png` (48x48)
  - `assets/icons/kant-128.png` (128x128)
  - `assets/icons/kant-256.png` (256x256)
  - `assets/icons/kant.png` (256x256 kopi til pixmap fallback)
  er genereret ud fra `assets/icons/kant.svg`.
- [x] **AC3 - Debian (.deb) Pakkekonfiguration**: `Cargo.toml` indeholder `assets` mappings for de faste PNG-ikoner i `usr/share/icons/hicolor/{48x48,128x128,256x256}/apps/kant.png` samt den universelle Pop!_OS fallback `usr/share/pixmaps/kant.png`.
- [x] **AC4 - Zero-Privilege Lokal Desktop Install CLI**: `kant --install-desktop` (eller `-i`) installerer eller opdaterer `~/.local/share/applications/kant.desktop`, `~/.local/share/icons/hicolor/scalable/apps/kant.svg` og `~/.local/share/pixmaps/kant.png` for den lokale bruger uden behov for root/sudo, og trigger opdatering af desktop/ikon-cachen.
- [x] **AC5 - Verifikation & Regression Gauntlet**: En integrationstest `tests/linux_desktop_integration.rs` validerer `application_id`, PNG-filernes integritet og dimensioner, samt desktop- og deb-konfigurationen.

## 🚫 Must NOT
- Må IKKE bryde Windows- eller macOS-byg (unconditional eller platform-gated `application_id`).
- Må IKKE kræve `sudo`/root rettigheder for lokal bruger-integration.
- Må IKKE forringe opstartstiden.

## 📝 Revisions
- 2026-10-15: Oprettet opgavepakke efter brugergodkendelse.

## 🧪 Verifikation
- `cargo test --test linux_desktop_integration`
- `cargo check --workspace`
- Kørsel af `--install-desktop` på det lokale Pop!_OS system.
