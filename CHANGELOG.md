# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.1] - 2026-10-28

### Added
- **Ortogonal Edge Routing & Bends**: Interaktiv manuel linjeføring med segment-dragging, frie knækpunkter, grid-snapping og rumlig forankring ved node-flytning på tværs af Begrebs- og Informationsmodellen (Task 096 & ADR 005).
- **Dynamisk Port-Sliding & Zonedrevet Portskift**: Terminale segmenter glider langs nodens kanter og skifter automatisk portzone (Top/Side/Bund) ved segment-træk forbi hjørner (Task 096).
- **Collinear Bend Pruning**: Automatisk fjernelse af overflødige knækpunkter og eliminering af mikrosegmenter og vinkelret overshoot ved linje-justering (Task 096).
- **Inspector Reset af Edge Routing**: Ny knap `[Gendan automatisk linjeføring]` i egenskabspanelet samt sletning af knækpunkter via dobbeltklik eller Delete/Backspace (Task 096).
- **Enkelt Palette Søgning & Combobox**: Strømlinet koncept- og klasse-palette med integreret tekstsøgning, optisk centrering og selektiv canvas-placering (Tasks 090, 092).
- **Moderne Auto-Hide Scrollbars**: Elegant flydende 6px scrollbar med auto-hide og optimeret margin-clearance på modelpaletten (Tasks 091, 095).

### Removed
- **Realtids-kollaboration & Relay Server**: Fjernet Axum WebSocket-relay og realtime synkroniseringsmotor til fordel for 100% lokal filintegritet og nul eksterne baggrunds-daemons (Task 093 & ADR 001).
- **Git Integration fra Kerneapplikationen**: Udfaset indbygget Git-styring til fordel for ekstern versionskontrol og suveræn lokal modelintegritet (Task 094).

### Fixed
- Løst klik-lag og hit-tolerance ved kantgreb, så linjesegmenter kan trækkes direkte uden forudgående selektion (Task 096).
- Rettet isolering af terminale segment-træk, så manipulering af et endestykke ikke utilsigtet flytter fjerne knækpunkter (Task 096).

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
