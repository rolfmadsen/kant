---
type: Task Package
title: "Task 070: Fokuserbare Knapper i Begrebseditoren og Modaler (Tastaturnavigation & Tab-cyklus)"
description: "Implementering af tastaturfokuserbare knapper i begrebseditoren og samtlige modalvinduer, så Tab og Shift+Tab navigerer til knapper med tydelig visuel fokusring og Enter/Mellemrum aktivering"
status: done
generated: { by: process:antigravity-task-init, at: "2026-10-04T13:41:00Z" }
tags: [ui, keyboard, focus, accessibility, a11y, buttons, modals, concept-editor]
---

# Task 070: Fokuserbare Knapper i Begrebseditoren og Modaler (Tastaturnavigation & Tab-cyklus)

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-10-04`

## 🎯 Formål
At gøre applikationens formularer og dialoger fuldt tilgængelige med tastaturet:
1. Gøre det muligt med <kbd>Tab</kbd> og <kbd>Shift+Tab</kbd> at navigere videre fra inputfelter til handlingsknapper (*Lokalt/Indlånt begreb*, *Annuller*, *Gem ændringer*, *Start Session*, *Klon*, *Gem Omslag* m.fl.).
2. Vise en tydelig, æstetisk FDA fokusring på den fokuserede knap jf. WCAG 2.4.7.
3. Tillade aktivering af fokuseret knap med <kbd>Enter</kbd> og <kbd>Mellemrum</kbd>.
4. Sikre hurtig-bekræftelse med <kbd>Enter</kbd> i modaler uden tekstfelter (f.eks. *Start Live Session (Vært)*), så man kan starte eller bekræfte direkte uden mus.

## 📋 Acceptance Criteria
- [x] **AC1 - Fokuserbare knapper i Begrebseditoren (`ConceptEditor`)**:
  - <kbd>Tab</kbd> navigerer gennem felterne og lander på:
    1. Domæne-knapperne (*Lokalt begreb (Ja)* / *Indlånt begreb (Nej)*).
    2. *Annuller*-knappen (`CancelConceptEdit`).
    3. *Gem ændringer* / *Opret begreb*-knappen (`SaveConcept`).
  - <kbd>Shift+Tab</kbd> navigerer baglæns i samme rækkefølge.
  - Knapperne viser en markant fokusring, når de har fokus.
  - <kbd>Enter</kbd> eller <kbd>Mellemrum</kbd> udløser knappens handling.
- [x] **AC2 - Fokuserbare knapper og Tab-cyklus i Modaler uden tekstfelter**:
  - I modaler uden tekstfelter (f.eks. `StartSessionModalState`, `ConflictResolverModalState`, `ExcelImportPreviewState`, `DiagnosticsModalState`, `ExcelSchemaGuideModal`):
    - Modalen har fra åbning eller ved første <kbd>Tab</kbd> fokus på den primære handlingsknap (*Start Session*, *Løs og fortsæt*, *Gennemfør import*, *Kopiér rapport*).
    - <kbd>Tab</kbd> cykler rundt mellem modalens knapper (f.eks. [Kopiér] -> [Annuller] -> [Start Session]).
    - <kbd>Enter</kbd> bekræfter den fokuserede knap (eller modalens primære handling).
    - <kbd>Esc</kbd> lukker modalen jf. Task 069.
- [x] **AC3 - Tab-cyklus til knapper i Modaler med tekstfelter**:
  - I modaler med tekstfelter (`MetadataModalState`, `JoinSessionModalState`, `GitCloneModalState`, `GitConnectionModalState`, `PublishModalState`, `FileDialogModal`, `RelationDialogState`, `QuickCreateState`):
    - Når sidste tekstfelt forlades med <kbd>Tab</kbd>, overgår fokus til modalens handlingsknapper (*Annuller* og *Gem/Klon/Forbind/Send*).
    - Efter sidste knap looper fokus tilbage til første inputfelt.
- [x] **AC4 - Visuel Fokusring & FDA Temaintegretion**:
  - Knapper med fokus styles med en tydelig 2px kontrastrig fokusring/ramme i overensstemmelse med `ThemeColors::PRIMARY` eller `ThemeColors::FDA_BORROWED_BLUE`.
- [x] **AC5 - Verifikation & Regression Gauntlet**:
  - Samtlige nye tastaturflows er dækket af automatiserede accepttests i `tests/acceptance.rs`.
  - Hele testsuiten (`cargo test --workspace`) og `cargo clippy --workspace -- -D warnings` passerer grønt.

## 🚫 Must NOT
- Må IKKE ændre eksisterende globale genveje som `Ctrl+S`, `Ctrl+B`, `Ctrl+1..3` eller zoom `Ctrl++` / `Ctrl+-`.
- Må IKKE bryde almindelig tekstindtastning i inputfelter (mellemrum skal altid skrive et mellemrumstegn, når et tekstfelt er aktivt).
- Må IKKE bryde den lagdelte <kbd>Esc</kbd>-afvikling etableret i Task 069.

## 📝 Revisions
- 2026-10-04: Oprettet opgavepakke efter brugergodkendelse.

## 🧪 Verifikation
- `cargo test --test acceptance test_task_070_`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
