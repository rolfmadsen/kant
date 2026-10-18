---
type: Knowledge Bundle Index
title: "Kant Context & Domain Glossary"
description: "Kernebegreber, arkitekturgrænser og definitioner for Kant"
status: stable
generated: { by: process:xgauntlet-init, at: "2026-09-06T18:00:00Z" }
tags: [glossary, domain-model, ubiquitous-language, okf]
---

# Kant Context & Domain Glossary

This document defines the core ubiquitous language for `Kant` using Aristotle's formula (*definitio per genus et differentiam*). It captures domain concepts without implementation noise and establishes the naming foundation for the codebase.

---

## 📖 Core Concepts

**Task**:
An executable unit of engineering work, that has bounded acceptance criteria and verifiable completion evidence.
_Avoid_: Ticket, issue, story, workitem.

**Layer**:
A verification step, that executes a specific analysis or testing command within a bounded timeout.
_Avoid_: Stage, phase, check-item.

**Gauntlet**:
A sequential verification pipeline, that executes verification layers with fail-closed semantics and halts on the first mandatory failure.
_Avoid_: Test runner, CI script, harness.

**Canonical Workspace Manifest**:
A deterministic SHA-256 digest, that captures normalized Git-tree/blob OID hashes and raw file contents across in-scope workspace paths.
_Avoid_: Git commit, workspace hash, checksum.

**Verification Report**:
An unsigned data record, that binds verification layer outcomes, diagnostic findings, and task contracts to the workspace manifest digests.
_Avoid_: Proof report, receipt, certification.

---

## 🏛️ FDA Modelleringsfaser (Metodisk Progression)

**Model (FDA Model)**:
En struktureret repræsentation af et forretnings- eller dataområde, der overholder de fællesoffentlige modelregler (FDA v2.1) for dokumentation, versionering og identifikation.
_Avoid_: Diagramfil, tegning, skitse.

**Begrebsliste (Phase 1 / Concept List)**:
En tabelformet repræsentation af et domænes begreber, der opstiller begreber med de 12 standardiserede metadatafelter jf. FDA Modelreglerne Bilag D og E.
_Avoid_: Ordliste, flad ordbog, begrebskatalog uden standardfelter.

**Begrebsmodel (Phase 2 / Concept Model)**:
En terminologisk grafmodel, der visualiserer begreber som noder og deres semantiske relationer (generaliseringer, del-hel og associationer) uafhængigt af datastrukturer og IT-systemer.
_Avoid_: Datamodel, entitetsdiagram, ER-diagram.

**Informationsmodel (Phase 3 / Information Model)**:
En UML-baseret applikationsmodel, der beriger begreber med klassestrukturer, attributter med standard primitive datatyper, kodelister og multipliciteter med henblik på dataudveksling og IT-implementering.
_Avoid_: Fysisk databasemodel, skema, tabeldesign.

---

## 🔍 Begrebsmodellering (ISO 704 & FDA Regler)

**Begreb (Concept)**:
En mental enhed eller abstraktion, der sammenfatter fælles karakteristika for en mængde af fænomener i virkeligheden, og som identificeres ved en stabil HTTP-URI.
_Avoid_: Dataelement, entitet (når der menes begreb), række.

**Term**:
En sproglig betegnelse i naturligt sprog for et givet begreb inden for et domæne.
_Avoid_: Variabelnavn, feltnavn.

**Foretrukken term**:
Den sproglige betegnelse, som en myndighed eller et fagligt fællesskab officielt har fastlagt som den primære repræsentant for et begreb.
_Avoid_: Navn, overskrift, label.

**Struktureret definition**:
En tekstuel forklaring af et begrebs betydning, udformet efter Aristoteles' formel med angivelse af et overordnet begreb (*genus proximum*) og en eller flere adskillende egenskaber (*differentia specifica*).
_Avoid_: Cirkulær definition, opremsende definition, negativ definition, uformel kommentar.

**Betydningskendetegn (Characteristic)**:
En egenskab eller et træk ved et fænomen, som anvendes til at afgrænse og definere et begreb.
_Avoid_: Attribut (når der menes begrebstræk).

**Overbegreb (Superordinate concept)**:
Et begreb med et snævrere omfang og færre betydningskendetegn end sine underbegreber, og som udgør *genus proximum* for disse.
_Avoid_: Superklasse (i ren begrebsmodellering).

**Underbegreb (Subordinate concept)**:
Et begreb med et bredere omfang og flere betydningskendetegn end sit overbegreb, og som arver alle overbegrebets kendetegn.
_Avoid_: Subklasse (i ren begrebsmodellering).

**Generalisering (Is-a relation)**:
En semantisk relation mellem et overbegreb og et underbegreb, hvor underbegrebet arver alle overbegrebets kendetegn og tilføjer mindst ét adskillende kendetegn.
_Avoid_: Nedarvning, reference, pointer.

**Partitiv relation (Del-hel relation)**:
En semantisk relation mellem en helhed og dens bestanddele eller delelementer.
_Avoid_: Komposition (i ren begrebsmodellering).

**Association (Begrebsassociation)**:
En semantisk relation mellem to begreber, der ikke er af generaliserende eller partitiv art, men som beskriver en faglig sammenhæng.
_Avoid_: Fremmednøgle, foreign key.

---

## 📐 Informationsmodellering (UML & FDA Regler)

**Klasse (Information Class)**:
En UML-konstruktion i en informationsmodel, der repræsenterer en objekttype afledt af et eller flere begreber med tilknyttede attributter og forretningsregler.
_Avoid_: Databasetabel, entitetstype.

**Attribut**:
En navngiven egenskab ved en klasse, defineret med en datatype og en multiplicitet.
_Avoid_: Kolonne, felt.

**Multiplicitet**:
En angivelse af det tilladte antal forekomster af en attribut eller en relations-ende (f.eks. `0..1`, `1`, `0..*`, `1..*`).
_Avoid_: Kardinalitet (når der specifikt menes intervallet i UML).

**Kodeliste / Enumeration**:
En afgrænset mængde af tilladte kodeværdier for en given attribut jf. FDA Tabel B.
_Avoid_: Opslagstabel, enum-streng.

**Begrebssporing (Concept Lineage)**:
En eksplicit, sporbar reference fra en klasse eller attribut i informationsmodellen tilbage til det ophavsbegreb i begrebsmodellen, som den realiserer.
_Avoid_: Løs kommentar, udokumenteret mapping.

---

## 🖥️ Kants UI & Studio Anatomi

**Studio**:
Et sammenhængende modelleringsmiljø i Kant dedikeret til en bestemt fase (Concept Studio, Concept Model Studio, Information Model Studio), sammensat af lærred, palet, inspektør og vejledningspanel.
_Avoid_: Fane, vindue, view.

**Lærred (Canvas)**:
Den interaktive flade for grafisk modellering, der understøtter zoom, pan, grafrendering og direkte manipulation via mus og tastatur.
_Avoid_: Tegnebræt, diagramwidget.

**Palet (Palette)**:
Værktøjspanelet til venstre, der udstiller oprettelige og eksisterende model-elementer med søgning og drag-and-drop.
_Avoid_: Værktøjskasse, sidemenu.

**Inspektør (Inspector)**:
Egenskabspanelet til højre, der viser og redigerer det aktuelt valgte elements metadata, kendetegn, attributter og relationer.
_Avoid_: Properties-boks, modal (når det er et panel).

**Vejledningspanel (Guidance Panel)**:
Et kontekstuelt hjælpepanel, der løbende evaluerer modellen mod FDA-reglerne (herunder Aristoteles-definitionstjek og navnekonventioner).
_Avoid_: Fejlliste, linter-dialog.

---

## 🔀 Graf & Routing Domæne

**Node**:
Et visuelt element på lærredet med en position, dimensioner og tilknyttede porte, der repræsenterer et begreb, en klasse eller en enumeration.
_Avoid_: Boks, firkant.

**Edge (Forbindelse / Relation)**:
En visuel kurve eller linjeforbindelse mellem to noders porte, der visualiserer en semantisk relation eller UML-association.
_Avoid_: Streg, pil.

**Port**:
Et fastlagt tilkoblingspunkt på en node (Top, Bund, Venstre, Højre), hvor en edge kan hæftes.
_Avoid_: Ankerpunkt, pin.

**Ortogonal Manhattan Routing**:
En ruteberegningsalgoritme, der udelukkende genererer retvinklede (horisontale og vertikale) segmenter mellem noder uden diagonale skæringer.
_Avoid_: Direkte linjeføring, elastik-routing.

**Port-Bus & Bundling**:
En routing-mekanisme, der samler parallelle forbindelser ved samme port i en kanal for at minimere visuel støj.
_Avoid_: Trådrod, overlappende linjer.

---

## 📦 Modeludveksling

**XMI (XML Metadata Interchange)**:
En standardiseret OMG XML-modeludvekslingsrepræsentation, der overfører UML-klasser, attributter, datatyper, relationer og stereotyper tapsfrit til modelleringsværktøjer som Sparx Enterprise Architect.
_Avoid_: Proprietært værktøjsformat, flad XML.

**SKOS (Simple Knowledge Organization System)**:
En W3C standardiseret RDF-datamodel, der repræsenterer kontrollerede begrebsordbøger og tesaurer med sproglige præfikser, definitioner og semantiske relationer (broader, narrower, related).
_Avoid_: Rå ordliste, uformel taksonomi.

**SHACL (Shapes Constraint Language)**:
Et W3C standardiseret sprog til validering af grafbaserede datastrukturer, der udtrykker strukturelle krav og restriktioner (NodeShapes og PropertyShapes med kardinaliteter og datatyper) over RDF-grafer.
_Avoid_: Uvalideret RDF-skema, uformelle dataregler.


