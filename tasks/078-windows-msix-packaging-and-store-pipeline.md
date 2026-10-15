---
type: Task Package
title: "Task 078: Windows MSIX Packaging og Microsoft Store Pipeline"
description: "Byg og pak Kant som MSIX desktop applikation i GitHub Actions med automatisk generering af Windows Store fliseaktiver, AppxManifest.xml og MakeAppx validering til direkte indsendelse i Microsoft Partner Center."
status: done
generated: { by: process:antigravity-task-init, at: "2026-10-15T12:45:00Z" }
tags: [windows, msix, packaging, microsoft-store, github-actions, ci-cd]
---

# Task 078: Windows MSIX Packaging og Microsoft Store Pipeline

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-10-15`
**Scope**: `assets/windows/`, `.github/workflows/release.yml`, `tests/msix_packaging.rs`, `docs/adr/014-windows-msix-packaging-and-microsoft-store-distribution.md`

## 🎯 Formål
Etablere en fuldautomatisk Windows MSIX-pakningspipeline til Microsoft Store (Partner Center) for Kant under uafhængig udvikler-identitet (`RolfMadsen.Kant`), så:
1. Kant pakkes som en moderne Desktop Bridge (Win32 FullTrust) app i MSIX-format.
2. Microsoft Store re-signerer pakken gratis ved indsendelse uden behov for eksterne kodesigneringscertifikater.
3. Windows Store fliseaktiver (44x44, 50x50, 150x150, 310x150, 620x300) genereres deterministisk fra vektorgrafik.
4. `.github/workflows/release.yml` producerer både standard `.zip` og Store-klar `.msix` ved tag-releases.

## 📋 Acceptance Criteria
- [x] `assets/windows/AppxManifest.xml` indeholder korrekt Partner Center identitet:
  - Identity Name: `RolfMadsen.Kant`
  - Publisher: `CN=6101A799-2390-4F81-88B1-08BC634BEC12`
  - PublisherDisplayName: `Rolf Madsen`
  - Package Family Name: `RolfMadsen.Kant_e0ys345em6abe`
  - Capability: `runFullTrust`
  - EntryPoint: `Windows.FullTrustApplication` med eksekverbar `kant.exe`
  - File association for `.kant` filer
- [x] Windows Store billedaktiver er til stede under `assets/windows/images/` med nøjagtige dimensioner:
  - `Square44x44Logo.png` (44x44 px)
  - `Square50x50Logo.png` (50x50 px, Store logo)
  - `Square150x150Logo.png` (150x150 px)
  - `Wide310x150Logo.png` (310x150 px)
  - `SplashScreen.png` (620x300 px)
- [x] Rust accepttest `tests/msix_packaging.rs` validerer manifest-integritet, XML-skemaelementer og billeddimensioner.
- [x] `.github/workflows/release.yml` bygger og udgiver `kant-windows-x86_64.msix` og SHA256 checksum uden at bryde den eksisterende `.zip` release.
- [x] Arkitektonisk beslutning dokumenteret i `docs/adr/014-windows-msix-packaging-and-microsoft-store-distribution.md`.

## 🚫 Must NOT
- Må IKKE fjerne eller forringe den eksisterende standalone `.zip`-udgivelse for Windows.
- Må IKKE ændre ved Kants offline first, Zero-Daemon og Zero Ambient Authority arkitektur i `src/`.
- Må IKKE kræve dyre tredjeparts kodesigneringscertifikater (MSIX udnytter Microsoft Store re-signing).

## 📝 Revisions
- 2026-10-15: Oprettet opgavepakke efter succesfuld registrering af MSIX-appidentitet i Partner Center.

## 🧪 Verifikation
- `cargo test --test msix_packaging`
- Inspektion af genereret `.msix` pakkestruktur og manifest.
