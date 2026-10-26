# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.0] - 2026-10-26

### Added
- Robust Excel-import tilpasset den officielle FDA Begrebslisteskabelon med automatisk valg af databærende faneblade (`Begrebsliste DA` vs `Begrebsliste DA+EN`) og fanevælger (Task 088).
- Udtrækning og valgfri overførsel af FDA Forretningsmetadata fra Excel til modelomslag (Task 088).
- Diagnostisk logning af Excel-indlæsning, arkvalg og afviste rækker under `[excel]` i System- og Grafikdiagnostik (Task 088).
- Synliggørelse af fletning ved Excel-import via målmodel-banner og indikator for uændrede begreber i den åbne model (Task 089).
- Ny mulighed for at oprette en helt ny model direkte ud fra det importerede Excel-dokument (`Opret som ny model i stedet`) (Task 089).
- Decomposed slices arkitektur for Canvas Studio, Informationsmodel og Begrebsmodel (Tasks 085-087).
- WAI 2.0 tilgængelige design tokens og tokenshåndhævelse (Tasks 081-084 & ADR 015).

### Fixed
- Løst fejl hvor data i Excel-rækker blev forvekslet med kolonneoverskrifter eller fejlagtigt filtreret bort som gentagne overskrifter (Task 088).

## [0.8.6] - 2026-10-16
### Added
- Rebranding af applikation fra Edge til Kant (Task 033 & ADR 009).
- E2EE klientside krypto, WebSocket netværkskanal og blind Axum relay server (`crates/kant-relay`) (Tasks 025-029).
- FDA begrebs- og informationsmodellering, UML klassediagram og canvas studio (Tasks 001-083).
- Windows 11 renderingoptimering, DX12 fast-path og MSIX packaging (Tasks 062, 065, 066, 078).
