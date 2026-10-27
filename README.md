# Kant

[![GitHub Release](https://img.shields.io/github/v/release/rolfmadsen/edge?color=blue&logo=github)](https://github.com/rolfmadsen/edge/releases/latest)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org)

**Kant** er en moderne, lynhurtig skrivebordsapplikation til FDA begrebs- og informationsmodellering bygget i [Rust](https://www.rust-lang.org) med den reaktive GUI-ramme [Iced](https://iced.rs).

Applikationen giver arkitekter og modelleringsfolk et visuelt lærred til opbygning af FDA-kompatible begrebs- og informationsmodeller med automatisk orthogonal relation-routing, live koncept-studio, multivalente egenskaber og robust JSON-persistens. Brugergrænsefladen overholder WAI 2.0 / WCAG AA tilgængelighedskrav via et type-sikkert design-tokens system og dekomponerede feature-slices ([ADR 015](docs/adr/015-accessible-design-tokens-and-decomposed-ui-slices.md)).

---

## 💾 Download installationsfiler

Installationspakker genereres automatisk ved hver ny version og findes under [GitHub Releases (Seneste version)](https://github.com/rolfmadsen/edge/releases/latest).

| Operativsystem | Arkitektur | Filformat | Beskrivelse | Direkte Download |
| :--- | :--- | :--- | :--- | :--- |
| 🐧 **Linux (Debian / Ubuntu / Mint / Pop!_OS)** | x86_64 | `.deb` | Fuld systeminstallation m. app-ikon & menu | [Hent `.deb` pakke](https://github.com/rolfmadsen/edge/releases/latest) |
| 🐧 **Linux (Alle distributioner)** | x86_64 | `.tar.gz` | Transportabel binær | [kant-linux-x86_64.tar.gz](https://github.com/rolfmadsen/edge/releases/latest/download/kant-linux-x86_64.tar.gz) |
| 🍏 **macOS** | Apple Silicon (M1/M2/M3/M4) | `.tar.gz` | Transportabel binær | [kant-macos-aarch64.tar.gz](https://github.com/rolfmadsen/edge/releases/latest/download/kant-macos-aarch64.tar.gz) |
| 🪟 **Windows** | x86_64 | `.zip` | Transportabel applikation (`kant.exe`) | [kant-windows-x86_64.zip](https://github.com/rolfmadsen/edge/releases/latest/download/kant-windows-x86_64.zip) |

*(macOS Intel og Linux ARM64 tilføjes ved efterspørgsel).*

---

## 🚀 Installations- og kørselsvejledning

### 🐧 Linux (x86_64)

#### Metode A: Debian / Ubuntu / Mint / Pop!_OS (`.deb` — Anbefalet)
Installerer Kant direkte i dit system og tilføjer programmet til din applikationsmenu med officielt logo:
1. Hent den seneste `.deb`-fil fra [GitHub Releases](https://github.com/rolfmadsen/edge/releases/latest).
2. Dobbeltklik på den hentede fil for at åbne den i dit Software Center, eller kør i Terminal:
   ```bash
   sudo apt install ./kant_*_amd64.deb
   ```
3. Start Kant fra dit skrivebords programstarter eller ved blot at skrive `kant` i en terminal.

#### Metode B: Transportabel arkiv (`.tar.gz` — Alle distributioner)
1. Hent `kant-linux-x86_64.tar.gz`.
2. Pak arkivet ud og kør programmet:
   ```bash
   tar -xzf kant-linux-x86_64.tar.gz
   chmod +x kant
   ./kant
   ```
3. **Systemafhængigheder:**
   Kant benytter Vulkan/OpenGL via `wgpu`. De fleste moderne desktop-distributioner har disse forudinstalleret. Ved en minimal installation:
   - **Ubuntu/Debian:** `sudo apt update && sudo apt install -y libwayland-client0 libx11-6 libxkbcommon0 libvulkan1`
   - **Fedora:** `sudo dnf install wayland-libs libX11 libxkbcommon vulkan-loader`

---

### 🍏 macOS (Apple Silicon: M1 / M2 / M3 / M4)

1. **Download:** Hent `kant-macos-aarch64.tar.gz` fra tabellen ovenfor.
2. **Pak ud:** Dobbeltklik på filen i Finder eller kør i Terminal:
   ```bash
   tar -xzf kant-macos-aarch64.tar.gz
   ```
3. **Placering:** Flyt den udpakkede `kant`-binær til din foretrukne mappe (f.eks. programmer eller `~/bin`).
4. **Gatekeeper (Vigtigt ved første kørsel):**
   Da Kant er et open source-projekt uden betalt Apple Developer-certifikat, vil macOS Gatekeeper advare om, at programmet kommer fra en uidentificeret udvikler.
   - **Nemmeste løsning:** Højreklik på `kant`-filen i Finder, vælg **Åbn**, og klik derefter på **Åbn** i advarselsdialogen.
   - **Alternativt via Terminal:** Fjern karantæne-flaget:
     ```bash
     xattr -d com.apple.quarantine ./kant
     ./kant
     ```

---

### 🪟 Windows (x86_64)

1. **Download:** Hent `kant-windows-x86_64.zip`.
2. **Pak ud:** Højreklik på `.zip`-filen og vælg **Udpak alle...**.
3. **Start applikationen:** Dobbeltklik på `kant.exe`.
4. **Windows SmartScreen (Ved første kørsel):**
   Hvis Windows Defender SmartScreen viser dialogen *"Windows beskyttede din pc"*:
   - Klik på **Flere oplysninger**.
   - Klik på knappen **Kør alligevel**.

---

## 🛠️ Byg fra kildekode (For udviklere)

Hvis du ønsker at bidrage til Kant eller bygge fra kildekoden, kræver det en fungerende Rust-installation.

### 1. Forudsætninger
- **Rust:** Installér via [rustup.rs](https://rustup.rs/) (Rust 1.80 eller nyere).
- **Linux systembiblioteker (kun på Linux build hosts):**
  ```bash
  sudo apt install -y build-essential pkg-config libx11-dev libxcursor-dev \
    libxrandr-dev libxi-dev libxinerama-dev libwayland-dev libxkbcommon-dev \
    libvulkan1 libasound2-dev
  ```

### 2. Klon og kør
```bash
git clone https://github.com/rolfmadsen/edge.git
cd edge

# Kør i udviklingstilstand
cargo run

# Byg optimeret release-binær
cargo build --release
# Resultatet findes i target/release/kant (eller kant.exe på Windows)
```

### 3. Testsuite
```bash
cargo test
cargo clippy -- -D warnings
```

---

## 🌿 Git Model-Integration & Asynkront Samarbejde

Kant har en indbygget, førsteklasses Git-integration designet specifikt til forretningsarkitekter og modelleringsfolk ([ADR 011](docs/adr/011-decomposed-model-persistence-and-git-integration.md)). Formålet er at opnå samme høje brugervenlighed og konfliktimmunitet som coArchi-pluginet til Archi – helt uden teknisk Git-jargon, rå terminalkommandoer eller uforståelige merge-konflikter (`<<<<<<< HEAD`).

### 1. Dekomponeret Modelformat (`.kant/`)
Når en model versionsstyres, gemmes den i et dekomponeret katalogformat:
- `.kant/metadata.json`: Overordnede modelmetadata (navn, version, ansvarlig myndighed mv.).
- `.kant/concepts/<uuid>.json`: Én fil pr. forretningsbegreb.
- `.kant/classes/<uuid>.json`: Én fil pr. informationsklasse.
- `.kant/relations/<uuid>.json`: Én fil pr. relation eller association.
- `.kant/diagrams/<uuid>.json`: Én fil pr. diagramvisning og canvas-layout.

**Fordele:**
- **95 % færre konflikter:** Fordi hvert begreb og hver klasse bor i sin egen fil, kan flere arkitekter arbejde uafhængigt i samme model uden nogensinde at ramme de samme linjer i Git.
- **Deterministisk serialisering:** Alle JSON-nøgler og relationer skrives i ensartet alfabetisk orden med pæn formatering. Det eliminerer "støj-ændringer" i versionshistorikken.

---

### 2. Arbejdsgange i Brugergrænsefladen (`Filer ▾`)

Samtlige versionsstyringsfunktioner er tilgængelige direkte under menuen **`Filer ▾`** i topbaren:

#### A. Klon en eksisterende model fra Git
Hvis din organisation allerede har et model-repository på GitHub, GitLab eller Azure DevOps:
1. Gå til **`Filer ▾`** → **`📦 Klon model fra Git...`**.
2. Indtast repositoryets URL (f.eks. `https://github.com/organisation/fda-model.git`).
3. Vælg en lokal destinationsmappe på din maskine.
4. Klik **Klon og åbn model**. Kant henter arkivet, konfigurerer automatisk forbindelsen og åbner modellen direkte på lærredet.

#### B. Forbind en lokal model til et centralt fjernlager
Hvis du har oprettet en model lokalt og vil dele den med kolleger:
1. Opret et **nyt tomt repository** på GitHub, GitLab eller Azure DevOps (uden README eller licens).
2. I Kant: Gå til **`Filer ▾`** → **`🌐 Git-forbindelse & Fjernlager...`**.
3. Indtast URL'en på det tomme repository samt dit **Git Brugernavn (`user.name`)** og **Git E-mail (`user.email`)**.
4. Klik **Gem forbindelse**. Kant opretter automatisk det lokale Git-arkiv, hvis det ikke allerede findes, og kobler det til fjernlageret.

#### C. Udgiv modelændringer
Når du har lavet ændringer i begreber, klasser eller diagrammer:
1. Gå til **`Filer ▾`** → **`🚀 Udgiv modelændringer...`** (eller klik på status-badget i topbaren).
2. Dialogen viser en automatisk opsummering af dine ændringer i letforståelige domænehændelser (🟢 *Oprettet*, 🟡 *Opdateret*, 🔴 *Fjernet*).
3. Tilføj en valgfri versionsnote til dine kolleger og klik **Bekræft og udgiv**.

#### D. Hent seneste ændringer & Semantisk 3-vejs fletning
Når kolleger har udgivet ændringer til det fælles repository:
1. Vælg **`Filer ▾`** → **`📥 Hent seneste ændringer`**.
2. Kant henter opdateringerne og udfører en intelligent **semantisk 3-vejs model fletning** i hukommelsen:
   - Ikke-modstridende ændringer flettes automatisk sammen.
   - Hvis to personer har ændret det samme felt på samme begreb samtidigt (f.eks. forskellig ordlyd i definitionen), åbner Kant en overskuelig dialog (**Visuel Konflikthåndtering**), hvor du med ét klik kan vælge, om lokal version eller serverens version skal gælde.
   - Der skrives aldrig rå Git-konfliktmarkører til disk.

#### E. Modelhistorik & Tidslinje ("Time Travel" Audit)
- Vælg **`Filer ▾`** → **`⏳ Modelhistorik & Tidslinje...`** for at se hele modellens historik præsenteret som en visuel tidslinje over hvem der har ændret hvad og hvornår.
- Du kan også inspicere historikken for et specifikt begreb eller en specifik informationsklasse baseret på elementets unikke UUID.

---

### 3. 🔐 Adgangskontrol, Rettigheder & Git-Identitet

For at undgå fejlmeddelelser som *"Permission denied"* eller *"Authentication failed"*, er her de vigtigste ting at vide om rettigheder og opsætning:

#### Hvad er Git Brugernavn og E-mail (`user.name` & `user.email`)?
- ⚠️ **Vigtigt: Dette er IKKE din adgangskode eller dit token!** Det er udelukkende den **forfatter-signatur**, som Git stempler dine ændringer med i historikken ("Udgivet af Fornavn Efternavn <email>").
- **Du må ALDRIG indsætte dit Personal Access Token eller adgangskode i `user.email`** – hvis du gør det, bliver dit hemmelige token skrevet synligt ind i modellens historik, og GitHub vil automatisk spærre det af sikkerhedshensyn.
- **`user.name`:** Dit fulde navn (f.eks. `Mette Hansen`) eller dit foretrukne kaldenavn på GitHub.
- **`user.email`:** Din almindelige arbejdsmail tilknyttet din GitHub-konto (f.eks. `mette@organisation.dk`).
- **Sådan tjekker du om du allerede har det sat op på din maskine:**
  Kør i en terminal:
  ```bash
  git config --global user.name
  git config --global user.email
  ```
  Hvis de allerede returnerer dit navn og e-mail, arver Kant disse automatisk. Du kan altid tilpasse dem specifikt for modellen i dialogen **`Git-forbindelse & Fjernlager`**.

---

#### Hvordan godkender man adgang til et privat repository (Autentifikation)?
Kant benytter systemets standard Git-installation. Hvis organisationens model ligger i et privat repository, skal Git have adgang via én af følgende metoder:

1. **HTTPS med Fine-grained Personal Access Token (Anbefalet Enterprise-standard):**
   - Benyt URL-formatet: `https://github.com/organisation/model-arkiv.git`.
   - **Hvor skal mit token bruges?** Tokenet er din **adgangskode (password)**, *ikke* din e-mail.
     - På Windows og macOS åbner systemets *Git Credential Manager* automatisk et browser-vindue første gang og husker dit login sikkert via Single Sign-On (SSO).
     - På Linux, eller hvis du arbejder i et miljø uden Credential Manager, kan du enten:
       a) Indsætte tokenet som adgangskode, når Git spørger efter password.
       b) Indlejre tokenet direkte i URL'en: `https://<DIT_TOKEN>@github.com/organisation/model-arkiv.git`.
   - **Sikkerhed: Sådan opretter du et Fine-grained Token med Least Privilege:**
     - Gå til GitHub: **Settings** → **Developer Settings** → **Personal Access Tokens** → **Fine-grained tokens** → **Generate new token**.
     - **Resource owner:** Vælg den organisation, der ejer model-arkivet.
     - **Repository access:** Vælg **Only select repositories** og vælg udelukkende dit model-repository (f.eks. `model-arkiv`). Dette sikrer, at tokenet *aldrig* kan tilgå organisationens øvrige kildekode.
     - **Permissions (Repository permissions):**
       - Sæt **Contents** til **`Access: Read and write`** (tillader klon, hentning og udgivelse af modelændringer).
       - *(Metadata: Read-only tilføjes automatisk af GitHub)*.
     - **Fordel ved HTTPS i Enterprise:** Port 443 er altid åben i alle virksomheds-firewalls, proxies og VPN-forbindelser.

2. **SSH-nøgle (Alternativ for tekniske udviklere):**
   - Benyt URL-formatet: `git@github.com:organisation/model-arkiv.git`.
   - Hvis du allerede har en SSH-nøgle på din computer (`~/.ssh/id_ed25519.pub`), skal den tilføjes under din profil på GitHub/GitLab (**Settings** → **SSH and GPG keys**).
   - Har du ikke en nøgle, genereres den med:
     ```bash
     ssh-keygen -t ed25519 -C "din-email@organisation.dk"
     ```
   - *Bemærk:* Nogle virksomheds-firewalls og offentlige VPNs blokerer port 22 (SSH). Hvis du oplever timeouts med SSH på arbejdsnetværket, skift til HTTPS.

#### Repository-tilladelser (Permissions)
- For at kunne **klone og hente opdateringer** (`Pull`) skal du mindst have **Læseadgang (Read)** til repositoryet.
- For at kunne **udgive modelændringer** (`Push / Publish`) skal du have **Skriveadgang (Write)** som Collaborator eller medlem af organisationens modellerings-team på GitHub/GitLab.

---

## 📄 Licens

Dette projekt er licenseret under de vilkår, der fremgår af [LICENSE](LICENSE).
