---
type: Task Package
title: "Task 075: Tastaturnavigation (Tab) for Relations-egenskaber i Begrebsdiagram og Informationsmodel"
description: "Fuldendt tastaturstyring (Tab/Enter) for kantelementer i Begrebsdiagram og Informationsmodel samt slet-knapper under klassens Tilknyttede Relationer."
status: done
generated: { by: process:antigravity-task-init, at: "2026-10-04T22:47:00Z" }
tags: [ui, keyboard, focus, accessibility, a11y, edge, inspector, concept-model, information-model]
---

# Task 075: Tastaturnavigation (Tab) for Relations-egenskaber i Begrebsdiagram og Informationsmodel

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-10-04`

## 🎯 Formål
Sikre 100% tastaturnavigation til samtlige interaktive kontroller for relationer:
1. **2. Begrebsdiagram (Valgt kant / `selected_edge`)**:
   - `advance_concept_inspector_focus` (eller via `advance_tab_focus`) håndterer Tab/Shift-Tab cyklus for kanten.
   - Cirkulær Tab-sekvens:
     `[edge_label_input]` ⇄ `[⇄ Vend]` ⇄ `[Association]` ⇄ `[Generalisering]` ⇄ `[Halv pil (rettet)]` (kun ved Association) ⇄ `[🗑️ Slet relation]`.
   - Orange fokusring (`ThemeColors::FOCUS_RING`) på samtlige fokuserede knapper og checkboxe.
   - <kbd>Enter</kbd> på fokuseret knap udfører handlingen (vender kilde/mål, opdaterer relationstype, toggler pil, sletter relation).
2. **3. Informationsmodel (Valgt kant / `selected_info_edge`)**:
   - `advance_info_inspector_focus` udvides til også at håndtere `selected_info_edge.is_some()`.
   - Cirkulær Tab-sekvens:
     `[info_edge_label_input]` ⇄ `[⇄ Vend]` ⇄ `[Association]` ⇄ `[Generalisering]` ⇄ `[Komposition]` ⇄ `[Dependency]` ⇄ `[Halv pil (rettet)]` (kun ved Association) ⇄ `[Kilde Multiplicitet]` ⇄ `[Mål Multiplicitet]` (hvis ikke Generalisering) ⇄ `[🗑️ Slet relation]`.
   - Multiplicitet-dropdowns omkranses af fokusring ved fokus, og <kbd>Enter</kbd> cykler `Multiplicity::PRESETS`.
   - <kbd>Enter</kbd> på fokuserede knapper udfører tilsvarende operationer.
3. **3. Informationsmodel > Klasse-inspektør (Tilknyttede Relationer)**:
   - Slet-knapperne (`🗑️`) i sektionen "Tilknyttede Relationer" tildeles knap-IDs (`INFO_INSPECTOR_REL_DELETE_START + r`) og medtages i `build_info_inspector_targets()`.
   - Orange fokusring vises ved fokus, og <kbd>Enter</kbd> sletter relationen via `Message::DeleteClassRelation`.
4. **Enter-fokus fra lærred**:
   - Et enkelt tryk på <kbd>Enter</kbd> med en kant markeret på lærredet placerer direkte tastaturfokus i associationsnavnet (svarende til index 0 i cyklussen).

## 📋 Acceptance Criteria
- [x] **AC1 - Begrebsdiagram Kant Tab-cyklus**: Tab cykler cirkulært mellem navnefelt, vend, relationstypeknapper, retnings-checkbox og slet-knap, og wrapper tilbage til navnefeltet uden at lække fokus.
- [x] **AC2 - Begrebsdiagram Kant Enter-aktivering**: Enter på en fokuseret kant-knap udfører handlingen (omvending, typeændring, sletning).
- [x] **AC3 - Informationsmodel Kant Tab-cyklus & Multipliciteter**: Tab cykler gennem alle kant-kontroller (inklusive multiplicitets-pick_lists hvis synlige), og Enter cykler multiplicitets-presets.
- [x] **AC4 - Informationsmodel Klasse Tilknyttede Relationer**: Slet-knapper for klassens relationer kan fokuseres med Tab og aktiveres med Enter.
- [x] **AC5 - Regression Gauntlet**: Samtlige nye flows verificeres med tests i `tests/acceptance.rs`, og `cargo test --workspace` samt `cargo clippy --workspace -- -D warnings` forbliver 100% grønne.

## 🚫 Must NOT
- Må IKKE fjerne eller ændre kant-markeringen ved Tab-navigation.
- Må IKKE bryde Escape-hierarkiet (Esc fravælger kanten og nulstiller kantspecifikt fokus).
- Må IKKE forstyrre normal tekstindtastning eller mellemrumstegn i `edge_label_input` / `info_edge_label_input`.
- Må IKKE udføre remote git push eller destruktive git resets.

## 📝 Revisions
- 2026-10-04: Oprettet opgavepakke efter brugerinitiativ om fuldendt Tab-cyklus for relationer på Begrebsdiagram og Informationsmodel samt tilknyttede relationer under klasse-inspektoren.
- 2026-10-04: Implementeret `concept_inspector_focus` for Begrebsdiagram kantelementer samt udvidet `info_inspector_focus` til relationer og `INFO_INSPECTOR_REL_DELETE_START` for klasserelationer. Tilføjet komplet fokusring-styling og Enter-tastaturhandlere (inkl. cykling af multipliciteter). Verificeret via 3 accepttests i `tests/acceptance.rs` og 100% grøn test-gauntlet over hele workspacet. Opgaven markeret som DONE.

## 🧪 Verifikation
- `cargo test --test acceptance test_task_075_`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
