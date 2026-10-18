# Agent Guidelines: Kant

Dette repository følger **Evidence-First Development & Clean Craftsmanship** metodikken.

---

## 📊 Standard Response HUD Protocol
Formatér altid toppen af samtlige synlige agent-svar med det transparente Cockpit Task HUD kort (maks. 5 linjer):
> ### 🛡️ [Task: <Task Title / ID>] `[<Task Type>: <Phase>]`
> **Status**: `Phase: <SPEC | RED | GREEN | REFACTOR | GAUNTLET | DONE>` | `Gauntlet: <PASS | FAIL | PENDING>` | `Git: <branch>@<oid> • <clean | dirty: N files>`
> **Progress**: `Criteria: X/Y [■■□□□]` | `Scope: <affected crates/paths>`
> **Links**: 📋 [Task](tasks/) • 📄 [Spec](spec.md) • 📖 [Glossary](CONTEXT.md) • 🏛️ [ADR](docs/adr/README.md) • 🧪 [Evidence](evidence.md)
> 💡 **Next Action:** <kort beskrivelse af næste umiddelbare handling>

---

## 💡 Intent-to-Task Sparringsprocedure (Idéfase)
Når en bruger henvender sig med et ustruktureret eller uformelt ønske, fungerer agenten som proaktiv sparringspartner gennem en 4-trins model før en formel opgavefil oprettes i `tasks/`:
1. **Formål & Afgrænsning**: Afdæk det reelle behov, kerneegenskaber og operationelle grænser (hvad skal løses, og hvad skal eksplicit udelades?).
2. **Invarianter & Must NOT**: Fastlæg negative begrænsninger og arkitektoniske barrierer, der under ingen omstændigheder må brydes (f.eks. Zero-Daemon, Zero Ambient Authority, ingen eksterne sockets eller utilsigtede afhængigheder).
3. **RED Test-hypotese**: Formuler en præcis hypotese om den observerbare fejl, regressionsrisiko eller manglende adfærd, som en fejlet accepttest skal påvise.
4. **ADR-triggere**: Vurder om ændringen introducerer irreversible trade-offs eller bryder eksisterende beslutninger i `docs/adr/`. Hvis en beslutning udfordres, skal en ny ADR formuleres.

---

## 🗂️ Task Management Protocol (`tasks/`)
1. **Curated Scope:** Hvert ikke-trivielt arbejdsstykke spores som en præcis markdown-fil i `tasks/<number>-<slug>.md`.
2. **Standard Task Structure (OKF v0.2 Compliant):**
   Alle opgaver i `tasks/` SKAL starte med Open Knowledge Format (OKF v0.2) YAML frontmatter for at overholde `check-spec` og `verify`:
   ```markdown
   ---
   type: Task Package
   title: "Task <number>: <Title>"
   description: "<Kort formålsbeskrivelse>"
   status: active
   generated: { by: process:antigravity-task-init, at: "<YYYY-MM-DDTHH:MM:SSZ>" }
   tags: [<feature-tags>]
   ---

   # Task <number>: <Title>

   **Status**: `ACTIVE`
   **Intent**: `🚀 NEW FEATURE` | `🐛 BUG FIX` | `🔄 REFACTOR` | `🔄 ENHANCEMENT`
   **Oprettet**: `YYYY-MM-DD`

   ## 🎯 Formål
   Konkret målsætning og afgrænsning.

   ## 📋 Acceptance Criteria
   - [ ] Eksekverbare kriterier med klare forventede inputs og outputs.

   ## 🚫 Must NOT
   - Negative begrænsninger og arkitektur-invarianter, der under ingen omstændigheder må brydes.

   ## 📝 Revisions
   - YYYY-MM-DD: Oprettet opgavepakke.

   ## 🧪 Verifikation
   - Konkrete testkommandoer til afprøvning og validering.
   ```
3. **Clean Session Handoffs:** En ny chat-session starter ved at læse den udpegede `tasks/<task>.md` og `CONTEXT.md`.
4. **No Memory Rot:** Afsluttede opgaver markeres `DONE` (i både frontmatter `status: done` og body `**Status**: DONE`) og forbliver frosne.

---

## 🔄 Core Development Loop
```text
SPEC / GRILL → (Human Approval) → RED → GREEN → REFACTOR → GAUNTLET → EVIDENCE
```

1. **SPEC / GRILL**: Konkrete eksekverbare kriterier i `tasks/<task>.md` og `spec.md`, afstemt med `CONTEXT.md`.
2. **RED**: Skriv fejlede accepttests først, og bevis at de fejler med den forventede årsag.
3. **GREEN**: Minimal implementation for at få testene til at passere.
4. **REFACTOR**: Oprydning i kode og modularitet, mens assertionerne forbliver frosne.
5. **GAUNTLET**: Kør multi-layer verifikation via `xgauntlet verify`:
   - Linters & Static Analysis
   - Type Checks & Kompilering
   - Acceptance & Unit Tests
   - Invariant & Spec Tests (`xgauntlet check-spec`)
   - Mutation Testing Gauntlet
6. **EVIDENCE**: Forsegl verifikationsrapport og evidens i `verification-report.json` og `evidence.md`.
7. **SESSION HANDOFF**: Vis `🏁 SESSION HANDOFF` kortet med starter-prompt til næste session.

---

## 🔒 Lokal TDD Phase Checkpoint Protokol
For at sikre sporbarhed, atomiske tilbagerulningspunkter og beskytte mod context rot, skal agenten udføre lokale git commits (`git add` og `git commit`) ved hver fase-overgang i TDD-løkken jf. xGauntlet Platform Invariants:
- `SPEC`: `task(<id>): initialize task specification and criteria`
- `RED`: `test(<id>): add failing acceptance test for <feature> [RED]`
- `GREEN`: `feat(<id>): implement minimal logic to satisfy test [GREEN]`
- `REFACTOR`: `refactor(<id>): clean up module boundaries and types [REFACTOR]`
- `DONE`: `chore(<id>): seal evidence and mark task DONE`

**Kritiske Invarianter:**
- Foretag ALDRIG remote publication handlinger (`git push`).
- Foretag ALDRIG destruktive reset handlinger (`git reset --hard` eller `git clean -f`).
- Alle commits forbliver strengt lokale checkpoints på udviklerens maskine.
