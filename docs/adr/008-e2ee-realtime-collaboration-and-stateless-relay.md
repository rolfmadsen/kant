---
type: Architectural Decision Record
title: 'ADR 008: E2EE Realtids-kollaborering og Stateless WebSocket Relay'
status: deprecated
tags: [architecture, adr, collaboration, e2ee, websocket, relay, privacy, git]
---

# 8. E2EE Realtids-kollaborering og Stateless WebSocket Relay

**Status**: `deprecated` (Fjernet til fordel for asynkront Git-samarbejde jf. ADR 011)  
**Dato**: `2026-09-20` (Deprekeret `2026-10-27`)  

## Kontekst
Kant anvender p.t. en lokal disk-baseret persistensmodel ([ADR 003](003-project-persistence-and-autosave.md)), hvor FDA-modeller gemmes lokalt som JSON og versionsstyres via Git. Brugere har et stærkt ønske om at kunne samarbejde synkront i realtid (f.eks. under faciliterede modelleringsworkshops på tværs af organisationer).

Samtidig er organisationerne underlagt strenge krav til databeskyttelse, GDPR og forretningshemmeligheder. En centraliseret cloud-database (som Firebase eller Supabase) ville bryde Kants Zero-Daemon og Zero Ambient Authority principper, skabe driftsomkostninger og møde modstand i offentlige og private sikkerhedsvurderinger.

## Beslutning

1. **Stateless, Blind Relay Server (`crates/edge-relay`)**:
   - Vi etablerer en uafhængig, minimalistisk Axum-baseret WebSocket relay i et Cargo Workspace (`crates/edge-relay`).
   - Relayen gemmer **nul data på disk** (100% in-memory) og fungerer udelukkende som en "blind byte-router" mellem klienter tilsluttet samme `RoomId`.
   - Relayen har ingen afhængighed af forretningsmodellerne (`ModelProject`, `Concept` osv.) og router udelukkende rå binære frames (`bytes::Bytes`).

2. **End-to-End Encryption (E2EE) på Klientsiden**:
   - Al data (både snapshots og mutationer) krypteres med en symmetrisk 256-bit nøgle (`ChaCha20-Poly1305`), FØR den afsendes til relayen.
   - Nøglen genereres lokalt af værten (Host) og sendes **aldrig** til relayen i HTTP-headere, URL-paths eller query parametre.
   - Hverken serveren, cloud-udbyderen eller mellemliggende netværk kan læse modeller, begreber eller diagrammer.

3. **Asymmetrisk Værtsansvar & Disklås for Gæster**:
   - **Værten (Host)** ejer Git-arbejdskopien og den lokale filsti (`model.edge.json`). Kun værten udfører autosave eller gemmer til disk.
   - **Gæsten (Guest)** modtager modellens tilstand i hukommelsen (RAM). Gæstens lokale autosave deaktiveres eksplicit for at forhindre overskrivning af gæstens egne lokale modelprojekter. Gæsten har mulighed for at vælge "Gem som kopi...", hvis modellen ønskes bevaret lokalt.

4. **Nul-konfiguration for Gæster via Sessionsbillet (Token)**:
   - Værten vælger relay-server (Koyeb cloud preset, intern organisations-URL eller lokal docker) og genererer en kompakt sessionskode:
     `edge:v1:<base64(server)>:<room_id>:<base64(key)>`
   - Gæsten indsætter koden i "Join Live Session", hvorefter Edge automatisk udpakker server, rum og dekrypteringsnøgle uden manuel konfiguration.

5. **Mutationer & Event-throttling**:
   - For at forhindre mætning af netværksbåndbredde og Iced event-loopet batches flytning af noder på lærredet ved `MouseReleased` eller throttles til maks 10–15 Hz.

## Konsekvenser

### Positive
- **Fuld Datasuverænitet & Sikkerhed:** E2EE garanterer, at selv ved hosting på offentlige PaaS-platforme (f.eks. Koyeb/Fly.io) forbliver modellens indhold utilgængeligt for tredjepart.
- **Nul Driftsbyrde:** Relayen kræver ingen database, diskplads eller backup; den kan genstartes vilkårligt uden datatab, da den autoritative kilde ligger på værtens maskine.
- **Fleksibel Udrulning:** Præcis samme kodebase kan udrulles on-premise i Docker/Kubernetes eller køre i skyen med 1-klik deploy.

### Trade-offs & Begrænsninger
- **Afhængighed af Værtens Tilstedeværelse:** Hvis værten lukker sin Edge-applikation uden overdragelse, afbrydes sessionen for gæsterne (håndteres ved at gæster kan gemme et lokalt snapshot før afbrydelse).
- **Konfliktmodel:** Simultan redigering håndteres i første version via Last-Write-Wins (LWW) og vært-autoritativ voldgift frem for tungere CRDT-strukturer.
