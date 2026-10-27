---
type: System Specification
title: Specification - Kant Architecture & Capabilities
description: Macro system architecture, philosophy, and invariants for Kant
status: active
generated: { by: process:xgauntlet-init, at: "2026-09-06T18:00:00Z" }
tags: [specification, architecture, invariants]
---

# Specification: Kant Architecture & Capabilities

## 🎯 Philosophy & Core Capabilities
1. **Clean Architecture & Autonomous Features**:
   - Autonome feature-moduler med klare ansvarsområder og veldefinerede grænseflader (Screaming Architecture).
2. **FDA Modelregler & Metodisk Progression**:
   - Understøttelse af Digitaliseringsstyrelsens regler for begrebs- og datamodellering (v2.1).
   - Tretrins arbejdsgang: Begrebsliste (tabel) -> Begrebsmodel (graf) -> Informationsmodel (klasser/attributter).
3. **The Elm Architecture (TEA) Desktop UI**:
   - Reaktiv og type-sikker brugerflade i Iced adskilt fra det rene domænelag.
4. **Deterministic Verification**:
   - Multi-layer verifikations-pipeline styret af deklarativ konfiguration (`gauntlet.toml`).
5. **Cross-Platform OS Integration**:
   - Native fildialoger (`rfd`), indlejrede applikationsikoner (`.ico`, `.svg`) og glidende rendering uden platforms-hiccups på Windows, macOS og Linux.

## 📐 Architecture & Feature Modules
- `src/features/model/`: Overordnet FDA modelcontainer og metadata (§06-§13).
- `src/features/concepts/`: Begreber, termer, synonymer og strukturerede definitioner (Bilag D & E).
- `src/features/concept_model/`: Begrebsmodel-graf med noder, generaliseringer og associationer.
- `src/features/information_model/`: Informationsmodel med klasser, attributter, standard datatyper og multipliciteter.
- `src/features/git/`: Git-baseret modelsamarbejde, versionskontrol og synkronisering.
- `src/ui/`: Iced shell, faner, tema og præsentationslogik.
- `tasks/`: Eksekverbare opgavepakker med acceptkriterier.
- `docs/adr/`: Arkitektoniske beslutningsreferater:
  - [ADR 001: Iced Desktop GUI Framework](docs/adr/001-iced-architecture.md)
  - [ADR 002: Begrebsliste CRUD & Domænemodel](docs/adr/002-concept-list-crud.md)
  - [ADR 003: Projektpersistens & Autosave](docs/adr/003-project-persistence-and-autosave.md)
  - [ADR 004: Interaktiv Begrebsmodel Canvas](docs/adr/004-concept-graph-canvas.md)
  - [ADR 005: Ortogonal Manhattan Routing & Port-Bus](docs/adr/005-orthogonal-edge-routing-and-port-bus.md)
  - [ADR 006: Informationsmodel & Begrebssporing](docs/adr/006-information-model-and-concept-traceability.md)
  - [ADR 007: Canvas Studio Paradigmet & UML Klassediagram](docs/adr/007-canvas-studio-paradigm-and-uml-class-canvas.md)
  - [ADR 008: E2EE Realtids-kollaborering & Relay (Deprecated)](docs/adr/008-e2ee-realtime-collaboration-and-stateless-relay.md)
  - [ADR 009: Rebranding fra Edge til Kant](docs/adr/009-rebranding-from-edge-to-kant.md)
  - [ADR 010: Cross-Platform Native Fildialoger](docs/adr/010-cross-platform-native-dialogs-and-os-packaging.md)
  - [ADR 011: Dekomponeret Model-Persistens & Git](docs/adr/011-decomposed-model-persistence-and-git-integration.md)
  - [ADR 012: Maskinlæsbar Modeludveksling XMI & RDF](docs/adr/012-machine-readable-model-interchange.md)
  - [ADR 013: Rendering Diagnostics & WGPU på Windows](docs/adr/013-windows-rendering-diagnostics-and-wgpu-restoration.md)
  - [ADR 014: Windows MSIX Packaging & Store Distribution](docs/adr/014-windows-msix-packaging-and-microsoft-store-distribution.md)
  - [ADR 015: Tilgængelige Design Tokens & Dekomponerede UI-Slices](docs/adr/015-accessible-design-tokens-and-decomposed-ui-slices.md)
  - [ADR Template: Ny beslutningsskabelon](docs/adr/template.md)

## 🚫 Must NOT (System Invariants)
- Må IKKE introducere skjulte runtime-afhængigheder eller udokumenterede baggrundsprocesser (Zero-Daemon).
- Må IKKE afhænge af platforms-specifikke kommandolinjeværktøjer som `zenity` til fildialoger; platformsintegration skal ske via native cross-platform biblioteker (`rfd`).
- Må IKKE omgå deklarative verifikationslag eller ignorere fejlede tests.
- Må IKKE sammenblande FDA forretningsregler og validering med UI-renderingswidgets.
- Må IKKE foretage utilsigtede remote publication kommandoer (`git push`).

## 🧪 Multi-Layer Verification Contracts
- [ ] 100% test pass rate på alle konfigurerede lag.
- [ ] 0 linter- og typechecker-advarsler.
