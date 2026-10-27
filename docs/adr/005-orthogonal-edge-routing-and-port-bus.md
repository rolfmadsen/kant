---
type: Architectural Decision Record
title: 'ADR 005: Deterministisk Ortogonal Edge-Routing, Port-Bus og Bro-krydsninger'
status: accepted
tags: [architecture, adr, canvas, iced, graph, uml, fda, routing, orthogonal, manhattan]
---

# 5. Deterministisk Ortogonal Edge-Routing, Port-Bus og Bro-krydsninger

**Status**: `accepted`  
**Date**: `2026-09-19` (Opdateret `2026-10-27` med Task 096 Addendum)  

## Context
I `edge` begrebsmodellen (graf-canvas) skal relationer (Generaliseringer og Associationer) fremstå professionelt og letlæseligt efter FDA Modelreglerne (Kapitel 5 og 7):
1. Relationer skal forbinde noder med rene ortogonale linjer (90-graders Manhattan-vinkler) frem for skrå direkte linjer.
2. Pilehoveder (lukkede hvide trekanter for generaliseringer mod superklassen) skal ramme nodens ydre kant eksakt og må aldrig skjules eller overlappes af nodens kasse.
3. Ifølge FDA vejledningen (linje 1474 & 1526) har generaliseringspile ingen vilkårlig fritekst-label, da pilens semantik i sig selv betyder "er en specialisering af". Associationer bærer associationsnavn.
4. Når noder flyttes tæt på hinanden, må pilen ikke blive mast sammen eller vende forkert.
5. Flere relationer på samme side af en node skal organiseres harmonisk (samme type samles i fælles anker/stamme jf. FDA generaliseringssæt-princippet, forskellige typer fordeles i parallelle, ikke-overlappende kanaler).
6. Krydsende linjer skal visuelt markeres med en lille bue ("bro" / line-jump) for at undgå forveksling med sammenkoblede relationer.

## Decision
1. **Deterministisk Geometrisk Routing-modul (`EdgeRouter`)**:
   - Vi opbygger en dedikeret, testbar matematisk modul i `src/ui/graph_canvas/` (eller som en afkoblet hjælpefunktion) til at udlede ortogonale ruter givet to noders rektangler og relationstype.
2. **Dynamisk Port-allokering & Hysterese ved Nærhed**:
   - Hver node har 4 porte: Top, Right, Bottom, Left.
   - For generaliseringer er udgangspunktet: Subklasse (Top) $\rightarrow$ Superklasse (Bund) jf. FDA princip om opadgående hierarki.
   - Hvis den vertikale afstand mellem modstående kanter er mindre end en defineret pilerum-tærskel ($D_{\text{min}} = 36\text{px}$), skifter porten til side-porte (Højre/Venstre) for at give plads til et rent U- eller C-formet ortogonalt forløb.
3. **Port-Bus & Multi-Relation Slots**:
   - Relationer af samme type på samme side deles om ankerpunktet (eller deler stamme/spids mod målet).
   - Relationer af forskellig type fordeles i symmetriske slots langs sidens kant (f.eks. $\pm 24\text{px}$ offset) og rutes i parallelle baner med fast afstand ($14\text{px}$).
4. **Præcis Bounding-Box Skæring for Pilehoveder**:
   - Pilehovedets spids forankres eksakt på målnodens yderkant ($x, y$), så spidsen rører kanten udefra.
   - Noden og pilehovedet sammensættes i render-løkken så hverken hvid fyldning eller kantstreg overlapper uhensigtsmæssigt.
5. **Krydsnings-Detektion med Linjebroer (Line Hops)**:
   - Horisontale og vertikale segmenter kryds-tjekkes; ved skæringspunkt indsættes en halvcirkelbue ($r = 4\text{px}$) på det horisontale segment.
6. **Brugertilpasset Linjeføring, Frie Knækpunkter og Port-Sliding (Task 096 Addendum - 2026-10-27)**:
   - Automatisk ortogonal ruteberegning fungerer som *default*.
   - En relation kan overstyres med en eksplicit liste af knækpunkter (`bends: Vec<BendPoint>`), som forankres rumligt på canvas ved node-flytninger.
   - For relationer med knækpunkter erstattes fast slot-allokering af dynamisk *port-sliding*: Porten glider langs nodens kant for at flugte direkte med det tilstødende knækpunkt.
   - Ren ortogonalitet (90-graders Manhattan-vinkler) og vinkelret pilehoveddocking bevares som en ufravigelig FDA-invariant.

## Consequences
- **Positive**:
  - Høj visuel læsbarhed og professionel diagramkvalitet efter FDA Modelreglerne.
  - Ingen sammenpressede eller usynlige pilehoveder ved træk af noder.
  - Testbar geometrisk logik med 100% determinisme uden komplekse eksterne afhængigheder.
  - Fuld brugerkontrol over diagramlayout uden at give køb på ortogonal standardisering.
- **Trade-offs**:
  - Mere beregningsarbejde under canvas-optegning; holdes performant ved letvægts 2D vektoraritmetik uden tunge A*-gitter-søgninger.
