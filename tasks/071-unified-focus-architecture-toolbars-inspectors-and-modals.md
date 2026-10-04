---
type: Task Package
title: "Task 071: Ensartet Fokusarkitektur for Værktøjslinjer, Inspektorer og Resterende Modaler"
description: "Etablering af en samlet standard for tastaturfokus (FocusCycle) på tværs af fanebladsværktøjslinjer, side-inspektorer og samtlige modaler samt sanering af ConceptEditorFocusTarget"
status: done
generated: { by: process:antigravity-task-init, at: "2026-10-04T16:58:00Z" }
tags: [ui, keyboard, focus, accessibility, a11y, buttons, modals, toolbars, inspector]
---

# Task 071: Ensartet Fokusarkitektur for Værktøjslinjer, Inspektorer og Resterende Modaler

**Status**: `DONE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-10-04`

## 🎯 Formål
At etablere en ensartet, robust fokus- og tastaturnavigationsstandard baseret på `FocusCycle` i tre lag, så brugeren konsistent kan navigere og aktivere alle interaktive elementer i hele applikationen:
1. **Sanering:** Fjerne redundant prototypekode (`ConceptEditorFocusTarget`) i `src/ui/concept_editor.rs` og anvende `crate::ui::focus::FocusCycle` som eneste fokus-state-machine i hele kodebasen.
2. **Værktøjslinje i Faneblad 1 (Begrebsliste):** Gøre knapperne *+ Nyt Begreb*, *Importér Excel*, *Eksportér Excel*, *Hent skabelon* og *Skemaguide* fokuserbare med den 1.5px orange fokusring (`ThemeColors::FOCUS_RING`) og aktiverbare med <kbd>Enter</kbd> / <kbd>Mellemrum</kbd>.
3. **Filer-menu modaler:** Sikre at *Filer > + Nyt projekt* initialiserer `modal_focus` på samme måde som *Modelomslag & Metadata...*, og at *Modelhistorik & Tidslinje* har fokus på "Luk"-knappen med orange fokusring.
4. **Samarbejde > Start Live Session (Vært):** Sikre at server-URL tekstfelt, preset-vælger og handlingsknapper indgår i den cirkulære fokuscyklus.
5. **Inspektorer i Begrebsdiagram & Informationsmodel (Egenskaber):**
   - Tilføje ID på *Definition* (`inline_concept_definition`) i lærredets hurtigredigering og forsyne *Annuller* og *Gem Begreb* med orange fokusring.
   - Forbinde Informationsmodellens Egenskabspanel til fokuscyklussen for klassenavn, beskrivelse og knapper.

## 📋 Acceptance Criteria
- [x] **AC1 - Sanering af parallel kode**:
  - `ConceptEditorFocusTarget` i `src/ui/concept_editor.rs` er saneret og erstattet af `crate::ui::focus::FocusTarget` / `FocusCycle`.
- [x] **AC2 - Faneblad 1 Værktøjslinje fokuscyklus**:
  - Når man er på faneblad 1 og ingen modal er åben, cykler <kbd>Tab</kbd> fra søgefeltet videre til *+ Nyt Begreb*, *Importér Excel*, *Eksportér Excel*, *Hent skabelon* og *Skemaguide*.
  - Hver fokuseret knap viser en 1.5px orange fokusring (`ThemeColors::FOCUS_RING`).
  - <kbd>Enter</kbd> eller <kbd>Mellemrum</kbd> på en fokuseret værktøjsknap aktiverer handlingen.
- [x] **AC3 - Filer > Nyt projekt & Historik modaler**:
  - *Filer > + Nyt projekt* (`Message::NewProject`) initialiserer `modal_focus` med alle 13 felter og sætter initialt fokus på *Modelnavn* (`meta_name`).
  - *Modelhistorik & Tidslinje* sætter fokus på "Luk"-knappen, der viser den orange fokusring og lukker ved <kbd>Enter</kbd>.
- [x] **AC4 - Samarbejde > Start Live Session**:
  - Modalen understøtter tastaturnavigation til eventuelt custom server-URL inputfelt samt de 3 handlingsknapper.
- [x] **AC5 - Egenskaber / Hurtigredigering i Faneblad 2 & 3**:
  - Begrebsdiagram Hurtigredigering: <kbd>Tab</kbd> fra *Foretrukken term* skifter til *Definition*, derefter til *Annuller* (orange ring), derefter *Gem Begreb* (orange ring), og looper tilbage til *Foretrukken term*.
  - Informationsmodel Egenskaber: Klassenavn og beskrivelse kan navigeres med Tab og handlingsknapper modtager fokus.
- [x] **AC6 - Regression Gauntlet**:
  - Samtlige nye flows verificeres med automatiserede accepttests i `tests/acceptance.rs`.
  - `cargo test --workspace` og `cargo clippy --workspace -- -D warnings` forbliver 100% grønne uden warnings.

## 🚫 Must NOT
- Må IKKE bryde almindelig tekstindtastning i aktive tekstfelter.
- Må IKKE ændre eksisterende globale genveje som `Ctrl+S`, `Ctrl+B`, `Ctrl+1..3` eller zoom.
- Må IKKE lave destruktive git-handlinger eller remote push.

## 📝 Revisions
- 2026-10-04: Oprettet opgavepakke efter brugerfeedback for at etablere en gennemgående, ensartet fokusarkitektur.

## 🧪 Verifikation
- `cargo test --test acceptance test_task_071_`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
