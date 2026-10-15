---
type: Architectural Decision Record
title: 'ADR 014: Windows MSIX Packaging og Microsoft Store Distribution for Uafhængig Udvikler'
status: accepted
tags: [architecture, adr, windows, msix, packaging, microsoft-store, ci-cd]
---

# 14. Windows MSIX Packaging og Microsoft Store Distribution for Uafhængig Udvikler

**Status**: `accepted`  
**Date**: `2026-10-15`  

## Context
Kant distribueres til Windows-brugere. Ved distribution til det brede publikum via Microsoft Store som individuel/uafhængig udvikler opstår et afgørende valg af pakke- og distributionsarkitektur:
1. **Traditionel Win32 (EXE/MSI)**: Microsoft Store tillader unpackaged Win32 apps via eksterne download-URL'er. Dette medfører imidlertid et ufravigeligt krav om, at installationsfilen og dens binærer skal være digitalt signeret med et kommercielt kodesigneringscertifikat (EV/OV) fra en rod-CA i Microsoft Trusted Root Program (en markant løbende omkostning og administrativ barriere for en individuel udvikler). Derudover hostes installationsfilerne ikke af Microsoft, automatiske baggrundsopdateringer skal styres af appen selv, og afinstallation efterlader ofte rester i filsystem og registreringsdatabase.
2. **Packaged Desktop Bridge (MSIX)**: Microsofts moderne containeriserede pakkeformat til Windows. Når en app indsendes som MSIX under en reserveret pakkeidentitet i Microsoft Partner Center, re-signerer Microsoft Store automatisk og vederlagsfrit pakken med Microsofts eget officielle rodcertifikat ved certificering. Pakken hostes på Microsoft Store CDN, opdateres automatisk i baggrunden via Windows Update, og udrulles i en sandboxed container med ren afinstallation.

## Decision
1. **Valg af MSIX som primært Store-pakkeformat**:
   - Kant pakkes som en MSIX-applikation (`RolfMadsen.Kant`) ved hjælp af Windows SDK's `MakeAppx.exe`.
   - Manifestet `assets/windows/AppxManifest.xml` konfigureres med `EntryPoint="Windows.FullTrustApplication"` og capability `runFullTrust`. Dette tillader den eksisterende Win32 Rust/Iced-binære (`kant.exe`) at køre med normal brugerintegritet uden modifikationer i kildekoden.
   - Transparent filsystemomdirigering i Windows Desktop Bridge sikrer, at Kants brug af standard `%APPDATA%\kant` til seneste modeller fungerer fejlfrit.
2. **Bevaring af standalone ZIP-distribution**:
   - Den eksisterende `.zip`-pakke (`kant-windows-x86_64.zip`) bevares fuldstændigt uændret i GitHub Releases til brugere, der ønsker at køre Kant portabelt uden om Microsoft Store.
3. **Deterministisk Flisegenerering og Automatiseret CI/CD**:
   - Påkrævede Store-fliseaktiver (`Square44x44Logo.png`, `Square50x50Logo.png`, `Square150x150Logo.png`, `Wide310x150Logo.png`, `SplashScreen.png`) genereres deterministisk fra vektorgrafikken `assets/icons/kant.svg`.
   - `.github/workflows/release.yml` udvides så Windows-buildet på `windows-latest` automatisk bygger og pakker `kant-windows-x86_64.msix` og genererer SHA256-checksum ved hvert tag-release.

## Consequences
- **Positive konsekvenser**:
  - Nul omkostning til dyre tredjeparts kodesigneringscertifikater; Microsoft Store signerer pakken gratis ved ingestion.
  - Ingen Windows SmartScreen advarsler for brugere, der installerer via Store.
  - 100% automatiske baggrundsopdateringer via Store-klienten.
  - Ingen ændringer påkrævet i Kants Rust-kildekode i `src/`.
  - Fuld bagudkompatibilitet for eksisterende brugere via fortsat distribution af portable ZIP-arkiver.
- **Trade-offs**:
  - Udvidelse af CI/CD-pipeline med `MakeAppx.exe` pakningstrin i release workflow.
