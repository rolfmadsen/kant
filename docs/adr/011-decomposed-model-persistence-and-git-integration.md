---
type: Architectural Decision Record
title: 'ADR 011: Dekomponeret Model-Persistens (.kant) og Semantisk Git Model-Integration'
status: accepted
tags: [architecture, adr, git, persistence, decomposed-storage, semantic-merge, grafico]
---

# 11. Dekomponeret Model-Persistens (.kant) og Semantisk Git Model-Integration

**Status**: `accepted` (Delvist ændret: Del 1 `.kant/` persistens er aktiv; Del 2 & 3 Git-integration og 3-vejs merge er udfaset og fjernet jf. Task 094)  
**Date**: `2026-09-27` (Opdateret `2026-10-27`)  

> [!NOTE]
> **Opdatering (Task 094 - 2026-10-27):**
> Den indbyggede Git-integration (Del 3 `GitBackend`/git service/modaler) og Del 2 (semantisk 3-vejs merge motor `merge_models`) er fuldstændig fjernet fra Kant for at reducere kompleksitet og fastholde en ren desktop-modelleringsarkitektur.
> Del 1 (**Dekomponeret Katalogsæt `.kant/`**) forbliver 100% aktiv og understøttes fuldt ud til ekstern versionsstyring.

## Context
Kant gemmer modeller i et monolitisk JSON-format (`model.kant.json`).
Når flere arkitekter og modellerere samarbejder asynkront via Git (eller fildeling), skaber et monolitisk filformat store udfordringer:
1. **Falske merge-konflikter**: Hvis to brugere redigerer to fuldstændig uafhængige begreber eller klasser, opstår der ofte rå Git-mergekonflikter (`<<<<<<< HEAD`), fordi ændringerne sker i samme array i samme JSON-fil.
2. **Uoverskuelige diffs**: Ikke-deterministisk formatering (vilkårlig nøglerækkefølge eller array-omrokeringer) forurener commits med unødvendige linjeændringer.
3. **Teknisk Git-friktion for forretningsfolk**: Ikke-tekniske modellerere skal ikke konfronteres med rå terminal-kommandoer, hash-koder eller kryptiske merge-fejlmeddelelser.

## Decision
1. **Dekomponeret Katalogsæt (.kant/ layout)**:
   - Modellen serialiseres i en deterministisk mappestruktur inspireret af GRAFICO:
     - `.kant/metadata.json`: Overordnede metadata (titel, version, domæne, status).
     - `.kant/concepts/<uuid>.json`: Én fil pr. forretningsbegreb.
     - `.kant/classes/<uuid>.json`: Én fil pr. informationsklasse.
     - `.kant/relations/<uuid>.json`: Én fil pr. relation/association.
     - `.kant/diagrams/<uuid>.json`: Én fil pr. diagramvisning (canvas-layouts og koordinater).
   - Deterministisk sortering: Samtlige JSON-nøgler sorteres alfabetisk, og under-arrays sorteres deterministisk (fx attributter efter id). Afsluttende newline `\n` og standard 2-spaces indentering.
   - Fuld bagudkompatibilitet: Enkeltstående `model.kant.json` filer indlæses fortsat transparent, og applikationen kan eksportere/synkronisere et snapshot.

2. **Semantisk 3-Vejs Model Merge Motor (`merge_models`)**:
   - I stedet for linjebaserede merges i Git benytter Kant en model-bevidst merge-motor: `merge_models(base, ours, theirs)`.
   - Uafhængige entiteter og forskellige felter på samme entitet fusioneres automatisk.
   - Sletning vs. redigering håndteres uden panics (herreløse relationer markeres, eller klassen bevares med bemærkning).
   - Reelle uafklarede feltkonflikter isoleres i hukommelsen og præsenteres i en visuel dialog ("Brug min version" vs "Brug serverens version"). Rå konfliktmarkører skrives aldrig til disk.

3. **Trait-baseret Git Backend Abstraktion (`GitBackend`)**:
   - For at sikre maksimal robusthed på tværs af Linux, Windows og macOS etableres et `GitBackend` trait.
   - Domæne-, storage- og merge-lag er 100% uafhængige af den konkrete Git-transport.
   - UI'et anvender udelukkende domænesprog ("Hent seneste", "Udgiv model", "Modelhistorik").

## Consequences
- **Positive konsekvenser**:
  - Op mod 95 % af alle potentielle merge-konflikter elimineres automatisk i Git, fordi samtidige ændringer rammer separate UUID-filer.
  - commits bliver mikroskopiske, rene og lette at revidere i Git.
  - Deterministisk serialisering garanterer idempotent skrivning (ingen støj i git diff).
  - Slutbrugere beskyttes mod korrupte modeller og rå Git-syntaksfejl.
- **Trade-offs**:
  - Et modelprojekt på disk består af flere mindre filer i stedet for én enkelt fil.
  - Kræver synkroniseret sletning af filer på disk, når entiteter slettes i modellen.
