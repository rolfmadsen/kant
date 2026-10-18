---
type: Architectural Decision Record
title: 'ADR 015: Tilgængelige Design Tokens og Dekomponerede UI-Slices'
status: accepted
tags: [architecture, adr, design-tokens, accessibility, wcag-aa, ui, decomposition, package-by-feature]
---

# 15. Tilgængelige Design Tokens og Dekomponerede UI-Slices

**Status**: `accepted`  
**Date**: `2026-10-18`  

## Context
Kants brugergrænseflade er bygget i Rust med den reaktive GUI-ramme Iced. I takt med applikationens hurtige evolution er UI-koden stødt på to fundamentale arkitektoniske udfordringer:

1. **Ad-hoc farvestyling og manglende tilgængelighedsgarantier**:  
   Styling er hidtil foregået via rå `Color::from_rgb` og `Color::from_rgba` instansieringer spredt vilkårligt på tværs af widgets og canvas-renderers. Dette medfører risiko for utilstrækkelig kontrast mellem tekst og baggrund samt manglende overholdelse af gældende tilgængelighedsstandarder (WAI 2.0 / WCAG AA). Som professionelt værktøj til den fællesoffentlige digitale arkitektur (FDA) skal Kant leve op til lovpligtige standarder for universelt design og visuel tilgængelighed.

2. **Monolitisk UI-struktur (`src/ui/app.rs`)**:  
   Hovedkomponenten `src/ui/app.rs` er vokset til over 11.500 linjer. Filen huser p.t. routing, topbar, sidepaneler, model-metadata, begrebseditor, canvas-interaktioner og detaljerede domæne-dialoger i en enkelt sammenfiltret Elm-model. Dette bryder princippet om enkelt ansvar (*Single Responsibility Principle*), gør paralleludvikling konfliktfyldt og vanskeliggør isoleret testbarhed.

## Decision
For at løse disse udfordringer etableres følgende bindende arkitekturretningslinjer for Kant:

1. **Type-sikre semantiske Design Tokens (`src/design_system/tokens.rs`)**:
   - Al direkte instansiering af rå RGB-farveværdier (`Color::from_rgb`, `Color::from_rgba`) i UI-komponenter forbydes.
   - Kant indfører et centralt, typesikkert design-tokens system under `src/design_system/tokens.rs`, der udstiller semantiske tokens for farver (`ColorTokens`), typografi (`TypographyTokens`) og afstande (`SpacingTokens`).
   - Forbuddet mod rå RGB-tokens håndhæves maskinelt via `forbidden_tokens` i `gauntlet.toml` og `xgauntlet check-architecture`.

2. **Ufravigelig WCAG AA Kontrast-invariant ($\ge 4.5:1$)**:
   - Enhver tekstfarve og ethvert interaktivt kontrolelement (ikoner, knapper, tilstande) skal have et kontrastforhold på mindst **$4.5:1$** mod sin baggrund.
   - Kontrastforholdet beregnes efter W3C's specifikation for relativ luminans:
     $$\text{Contrast Ratio} = \frac{L_1 + 0.05}{L_2 + 0.05}$$
   - Kontrastkravet verificeres matematisk via dedikerede enhedstests, og der tillades ingen udokumenterede undtagelser.

3. **Modulopdeling efter `Package-by-Feature`**:
   - UI-komponenter og tilhørende tilstandshåndtering dekomponeres fra monolitten `src/ui/app.rs` ud i vertikale, isolerede feature-slices under `src/features/<feature>/ui/` (f.eks. `src/features/concepts/ui/`, `src/features/concept_model/ui/`, `src/features/information_model/ui/`).
   - Hver udskilt fil skal overholde Kants produktionsloft på højst 800 linjer (`max_file_lines = 800`).
   - `src/ui/app.rs` reduceres til en tynd overordnet applikations-shell, der udelukkende håndterer global vindueskonfiguration, top-level routing og fane-dispatch.

## Consequences
- **Positive konsekvenser**:
  - Garanteret overholdelse af WCAG AA kontrastkrav på tværs af hele applikationen.
  - Høj visuel ensartethed og forbedret vedligeholdbarhed med et formaliseret design-tokens system.
  - Elimineret risiko for regressioner via automatiseret arkitektur-linter og release-gatekeeper.
  - Drastisk reduceret kompleksitet og færre merge-konflikter, når `src/ui/app.rs` dekomponeres til selvstændige feature-moduler.
- **Trade-offs**:
  - Engangsindsats forbundet med at migrere eksisterende UI-kode til de nye tokens (udføres i Task 082).
  - Kræver disciplineret opsplitning af tilstande og beskeder mellem feature-moduler og shell (udføres i Task 083).
