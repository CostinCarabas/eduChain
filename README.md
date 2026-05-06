# EduChain PoC — Sistem de Certificare Educațională pe Blockchain

> **Blockchain-powered Educational Certificate System** construit pe **MultiversX**

[![Network](https://img.shields.io/badge/Network-MultiversX%20Devnet-green)](https://devnet-explorer.multiversx.com)
[![Rust](https://img.shields.io/badge/Rust-1.78%2B-orange)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-lightgrey)](#)

---

## Cuprins

1. [Descriere generală](#1-descriere-generală)
2. [Arhitectura sistemului](#2-arhitectura-sistemului)
3. [Contracte smart](#3-contracte-smart)
4. [Instalare și configurare](#4-instalare-și-configurare)
5. [Demo interactiv](#5-demo-interactiv)
6. [Referință CLI](#6-referință-cli)
7. [Schema metadate off-chain](#7-schema-metadate-off-chain)
8. [Testare](#8-testare)
9. [CI/CD](#9-cicd)
10. [Sandbox local](#10-sandbox-local)
11. [Securitate](#11-securitate)
12. [Conformitate și standarde](#12-conformitate-și-standarde)
13. [Performanță și costuri gas](#13-performanță-și-costuri-gas)
14. [Roadmap TRL](#14-roadmap-trl)
15. [Documentație suplimentară](#15-documentație-suplimentară)
16. [Referințe](#16-referințe)

---

## 1. Descriere generală

EduChain este un sistem de certificare educațională bazat pe blockchain care permite instituțiilor să emită diplome și certificate tamper-proof sub formă de **Soulbound Token (SBT)** pe rețeaua MultiversX. Fiecare certificat include un hash SHA-256 al metadatelor complete stocate off-chain, garantând integritatea criptografică fără a expune date personale on-chain.

### Problema rezolvată

Certificatele educaționale clasice sunt vulnerabile la:
- Falsificare și contrafacere
- Pierdere sau deteriorare fizică
- Procese lente și costisitoare de verificare
- Lipsa interoperabilității între instituții și angajatori

### Soluția EduChain

| Caracteristică | Implementare |
|---|---|
| **Imutabilitate** | Certificatele sunt stocate pe blockchain MultiversX — nu pot fi alterate |
| **Verificabilitate instantă** | Orice terț poate verifica autenticitatea unui certificat fără intermediari |
| **Confidențialitate** | Datele personale rămân off-chain; on-chain se stochează doar hash-ul SHA-256 |
| **Non-transferabilitate** | Certificatele sunt SBT — nu pot fi transferate între wallet-uri |
| **Revocare controlată** | Soft-revoke: NFT-ul rămâne în wallet, statusul devine `Revoked` on-chain |
| **Recompense tokenizate** | Studenții primesc token-uri `$ECT` pentru realizări academice |
| **Mentoring integrat** | Plăți pentru sesiuni de mentoring prin escrow smart contract |

### Contracte deployate pe Devnet

| Contract | Adresă | Explorer |
|---|---|---|
| `edu-chain-nft` | `erd1qqqqqqqqqqqqqpgqtgc2tn999va6cpmgujcv3jk4dt7wgam3lmpq6qakvu` | [🔗](https://devnet-explorer.multiversx.com/accounts/erd1qqqqqqqqqqqqqpgqtgc2tn999va6cpmgujcv3jk4dt7wgam3lmpq6qakvu) |
| `ect-token` | `erd1qqqqqqqqqqqqqpgqvyz832uc7pfan2usyqq8ek9k8vf7xhtvlmpqsf73v7` | [🔗](https://devnet-explorer.multiversx.com/accounts/erd1qqqqqqqqqqqqqpgqvyz832uc7pfan2usyqq8ek9k8vf7xhtvlmpqsf73v7) |
| `ect-escrow` | `erd1qqqqqqqqqqqqqpgq08hh9e5t57llh8p300hfdneqlxj80shzlmpq4f6qd6` | [🔗](https://devnet-explorer.multiversx.com/accounts/erd1qqqqqqqqqqqqqpgq08hh9e5t57llh8p300hfdneqlxj80shzlmpq4f6qd6) |
| `ect-anchor` | `erd1qqqqqqqqqqqqqpgqcy8ws6k9fthwfa9g67ksynvcfen7rdmglmpqpc7cl7` | [🔗](https://devnet-explorer.multiversx.com/accounts/erd1qqqqqqqqqqqqqpgqcy8ws6k9fthwfa9g67ksynvcfen7rdmglmpqpc7cl7) |
| Token `$ECT` | `ECT-f90acc` | [🔗](https://devnet-explorer.multiversx.com/tokens/ECT-f90acc) |
| Colecție NFT | `EDUCERT-174aad` | [🔗](https://devnet-explorer.multiversx.com/nfts/EDUCERT-174aad-01/transactions) |

---

## 2. Arhitectura sistemului

### 2.1 Structura proiectului

```
eduChain-PoC/
├── contracts/
│   ├── edu-chain-nft/          # Contract SBT certificate NFT
│   │   ├── src/
│   │   ├── scenarios/          # Teste scenario (.scen.json)
│   │   ├── tests/              # Runner teste Rust
│   │   ├── meta/               # Build tool multiversx-sc-meta
│   │   └── wasm/               # WASM build output
│   ├── ect-token/              # Contract token fungibil $ECT
│   ├── ect-escrow/             # Contract escrow sesiuni mentoring
│   └── ect-anchor/             # Contract registru hash-uri
├── interactor/                 # CLI Rust (clap v4) — deploy & interacțiune
│   ├── src/
│   │   ├── interactor_main.rs  # Entry point CLI
│   │   ├── config.rs           # KeySource (Env / File / Vault)
│   │   ├── state.rs            # Persistență adrese contracte (state.toml)
│   │   ├── nft.rs              # Comenzi NFT certificate
│   │   ├── token.rs            # Comenzi $ECT token
│   │   ├── escrow.rs           # Comenzi escrow
│   │   └── anchor.rs           # Comenzi ancorare hash
│   ├── fixtures/               # Fixture JSON certificate de test
│   ├── config.toml             # Configurație gateway, key source
│   └── state.toml              # Adrese contracte deployate
├── docs/
│   ├── schemas/                # Schema JSON-LD metadate off-chain (v1.0)
│   ├── RUNBOOK_OPS.md          # Runbook operațional
│   └── EduChain_TRL4_E2_Validation_Report.md
├── sandbox/                    # Docker Compose — chain-simulator local
│   ├── docker-compose.yml
│   ├── Dockerfile.interactor
│   └── mock-backend/server.js  # Mock server Node.js (port 3001)
├── demo/
│   ├── demo.sh                 # Script demo interactiv
│   └── demo-gui.html           # Interfață grafică demo pentru stakeholderi
└── .github/workflows/
    ├── ci.yml                  # fmt + clippy + test + WASM build
    └── devnet.yml              # Deploy nightly pe Devnet + E2E
```

### 2.2 Diagrama fluxului de date

```
┌─────────────────────────────────────────────────────────────────┐
│                         ACTORI                                  │
│                                                                 │
│   🏫 Instituție        🎓 Student       🔍 Verificator          │
│   (Operator/Issuer)    (Wallet MVX)     (Angajator/Terț)        │
└────────────┬──────────────────┬──────────────────┬─────────────┘
             │                  │                  │
             ▼                  ▼                  ▼
┌─────────────────────────────────────────────────────────────────┐
│              rust-interact (CLI Interactor Rust)                │
│         MultiversX Rust SDK · clap v4 · sha2 crate             │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
          ┌──────────────────────────────────────┐
          │  MultiversX Devnet                   │
          │  gateway: devnet-gateway.mvx.com     │
          └──────────┬───────────────────────────┘
                     │
         ┌───────────┼───────────┬───────────────┐
         ▼           ▼           ▼               ▼
   ┌──────────┐ ┌─────────┐ ┌─────────┐ ┌──────────┐
   │edu-chain │ │ect-token│ │ect-escro│ │ect-anchor│
   │   -nft   │ │  $ECT   │ │   -w    │ │  hash    │
   │  SBT NFT │ │Treasury │ │ Escrow  │ │ registry │
   └────┬─────┘ └─────────┘ └─────────┘ └──────────┘
        │
        │  SHA-256(canonical JSON-LD) stocat on-chain
        │
        ▼
   ┌────────────────────────────────────────────────┐
   │   Off-chain Storage (IPFS / Backend DB)        │
   │   cert1.metadata.jsonld · Open Badge 3.0       │
   └────────────────────────────────────────────────┘
```

### 2.3 Dependențe între contracte

```
ect-token ──(ect_token_id)──► edu-chain-nft
ect-token ──(ect_token_id)──► ect-escrow
                               ect-anchor   (standalone — fără dependențe)
```

### 2.4 Fluxul de emitere certificat

```
1. Instituție completează fixtures/cert.json
       │
       ▼
2. Interactor citește JSON-LD off-chain și sortează cheile (canonicalizare)
       │
       ▼
3. SHA-256(canonical JSON-LD) → content_hash (32 bytes)
       │
       ▼
4. issueCertificate(student_addr, attrs{content_hash, ...}) → TX pe Devnet
       │
       ▼
5. SBT NFT creat cu nonce unic → transferat automat în wallet-ul studentului
       │
       ▼
6. Studentul poate partaja nonce-ul cu orice verificator
       │
       ▼
7. Verificatorul apelează verifyCertificate(nonce) → CertificateAttributes
```

---

## 3. Contracte smart

### 3.1 `edu-chain-nft` — Certificate SBT

Contractul principal de emitere certificate. Fiecare certificat este un **Soulbound Token** (NFT non-transferabil din perspectiva aplicației) cu atributele stocate on-chain și hash-ul SHA-256 al metadatelor complete.

**Endpoint-uri:**

| Endpoint | Caller | Descriere |
|---|---|---|
| `init` | deployer | Setează owner-ul; niciun token emis inițial |
| `issueToken(name, ticker)` | owner | Creează colecția SBT ESDT (ex: EDUCERT) |
| `setLocalRoles` | owner | Activează rolul de creare NFT |
| `addIssuer(addr)` | owner | Adaugă adresă în whitelist-ul de emitere |
| `removeIssuer(addr)` | owner | Elimină emitent din whitelist |
| `isIssuer(addr)` | oricine (view) | Verifică apartenența la whitelist |
| `issueCertificate(student, name, attrs)` | issuer | Mintează 1 SBT → transferat studentului |
| `issueCertificateBatch(list)` | issuer | Batch: până la 50 certificate / tranzacție |
| `revokeCertificate(nonce)` | issuer | Soft-revoke (setează flag `revoked`) |
| `verifyCertificate(nonce)` | oricine | Returnează `CertificateAttributes` + status |

**Câmpuri `CertificateAttributes` (on-chain):**

| Câmp | Prioritate | Descriere |
|---|---|---|
| `content_hash` | P0 | SHA-256 al JSON-LD canonic off-chain |
| `student_address` | P0 | Adresă bech32 wallet student |
| `institution_name` | P0 | Numele instituției emitente |
| `program_name` | P0 | Denumirea programului/cursului |
| `course_id` | P0 | Identificator curs |
| `achievement_type` | P0 | Course / Bootcamp / Certification / etc. |
| `instructor_name` | P0 | Instructor principal |
| `language` | P0 | ISO 639-1 (ex: `ro`, `en`) |
| `creation_timestamp` | P0 | UNIX timestamp emitere |
| `expiration_timestamp` | P0 | UNIX timestamp expirare (0 = permanent) |
| `eqf_level` | P0 | Nivel EQF (1–8) |
| `verification_url` | P0 | URL verificare metadate off-chain |
| `name` | P1 | Numele studentului (opțional, pseudonim recomandat) |
| `grade` | P1 | Notă numerică |
| `ects_credits` | P1 | Credite ECTS |

**Evenimente emise:**

```rust
CertificateIssued { nonce: u64, student: ManagedAddress, course_id: ManagedBuffer }
CertificateRevoked { nonce: u64, issuer: ManagedAddress }
```

---

### 3.2 `ect-token` — Token fungibil $ECT

Contractul de trezorerie pentru token-ul de recompensă `$ECT` (EduChain Token). Permite instituțiilor să distribuie recompense tokenizate studenților pentru realizări academice.

**Endpoint-uri:**

| Endpoint | Caller | Descriere |
|---|---|---|
| `init` | deployer | Inițializare |
| `issueToken(name, ticker, supply)` | owner | Emite token ESDT fungibil `$ECT` (o singură dată) |
| `addDistributor(addr)` | owner | Whitelist distribuitor recompense |
| `removeDistributor(addr)` | owner | Elimină distribuitor |
| `mintRewards(amount)` | owner | Mintează ECT în trezorerie |
| `addReward(beneficiary, amount)` | distributor | Adaugă recompensă pending pentru un student |
| `claimRewards` | beneficiary | Transferă pending rewards → caller |
| `payP2p(receiver, amount)` | owner | Plată directă din trezorerie |
| `treasuryBalance` | oricine (view) | Sold curent trezorerie |
| `pendingBalance(addr)` | oricine (view) | Recompense pending ale unei adrese |

**Flux recompense:**

```
Owner/Distribuitor → addReward(student, 500 ECT) → pending_balance[student] += 500
Student → claimRewards() → 500 ECT transferați în wallet
```

---

### 3.3 `ect-escrow` — Escrow sesiuni mentoring

Contractul de escrow care gestionează plățile pentru sesiunile de mentoring. Fondurile sunt blocate la creare și eliberate mentorului doar după confirmarea studentului.

**Stările unei sesiuni:**

```
            creare
              │
           [Open]
          /        \
    confirmare    expirare deadline
         │              │
    [Completed]    [Expired] → student refundat
          
    sau dacă dispute:
    [Disputed] → owner resolve → fonduri la winner
```

**Endpoint-uri:**

| Endpoint | Caller | Descriere |
|---|---|---|
| `init(ect_token_id)` | deployer | Setează token-ul acceptat |
| `createSession(mentor, deadline)` | student | Blochează $ECT, creează sesiunea |
| `confirmCompletion(session_id)` | student | Eliberează fonduri mentorului |
| `disputeSession(session_id)` | student/mentor | Marchează sesiunea ca disputată |
| `resolveDispute(session_id, winner)` | owner | Transferă fonduri câștigătorului |
| `expireSession(session_id)` | oricine | Returnează fonduri după deadline |
| `getSession(id)` | oricine (view) | Returnează struct `Session` |

---

### 3.4 `ect-anchor` — Registru hash-uri de conținut

Contract standalone pentru stocarea imutabilă a hash-urilor SHA-256 ale documentelor. Oferă dovadă de existență (*proof of existence*) a oricărui document la un moment dat.

**Endpoint-uri:**

| Endpoint | Caller | Descriere |
|---|---|---|
| `init` | deployer | Inițializare |
| `anchorContent(hash_32, metadata_uri)` | oricine | Stochează hash dacă nu e duplicat |
| `verifyAnchor(hash_32)` | oricine (view) | Returnează `OptionalValue<Anchor>` |
| `listAnchorsByAuthor(addr, from, limit)` | oricine (view) | Listă paginată anchore per autor |
| `revokeAnchor(hash_32, reason)` | autorul original | Soft-revoke ancoră |

**Struct `Anchor` (on-chain):**

```rust
Anchor {
    author:       ManagedAddress,  // Cine a ancorat
    timestamp:    u64,             // Block timestamp
    metadata_uri: ManagedBuffer,   // URI document off-chain
    revoked:      bool,            // false implicit
}
```

**Garanții:**
- Duplicate hash-uri sunt respinse (guard idempotență)
- Timestamp-ul este setat de blockchain — nu poate fi falsificat
- Revocarea este soft — înregistrarea rămâne vizibilă cu flag `revoked=true`

---

## 4. Instalare și configurare

### 4.1 Prerequisite

| Instrument | Versiune minimă | Instalare |
|---|---|---|
| Rust stable | ≥ 1.78 | `rustup toolchain install stable` |
| wasm32 target | — | `rustup target add wasm32-unknown-unknown` |
| multiversx-sc-meta | 0.65.x | `cargo install multiversx-sc-meta --locked` |
| Docker + Compose | v2 | [docs.docker.com](https://docs.docker.com/get-docker/) |
| mxpy (opțional) | ≥ 9 | `pip install multiversx-sdk-cli` |

### 4.2 Clonare și build

```bash
# 1. Clonare repository
git clone https://github.com/your-org/educhain-poc.git
cd educhain-poc

# 2. Build workspace Rust (debug)
cargo build --workspace

# 3. Build binar interactor (release)
cargo build --release -p rust-interact
# Binar la: target/release/rust-interact

# 4. Build artefacte WASM pentru toate contractele
for c in contracts/edu-chain-nft contracts/ect-token contracts/ect-escrow contracts/ect-anchor; do
  (cd "$c/meta" && cargo run -- build --no-wasm-opt)
done
```

### 4.3 Configurare cheie de semnare

Interactorul suportă trei surse de cheie, configurate în `interactor/config.toml`:

**Variabilă de mediu (recomandat pentru CI/CD):**
```bash
export EDUCHAIN_PEM="$(cat ~/wallets/devnet_operator.pem)"
```
```toml
# interactor/config.toml
[key]
source  = "env"
env_var = "EDUCHAIN_PEM"
```

**Fișier PEM local (dev):**
```toml
[key]
source    = "file"
file_path = "/absolute/path/to/wallet.pem"
```

**HashiCorp Vault (producție — viitor):**
```toml
[key]
source     = "vault"
vault_path = "secret/educhain/devnet-operator"
```

> ⚠️ **Niciodată** nu commite fișiere `.pem` sau conținutul lor în git.

### 4.4 Configurare rețea

```toml
# interactor/config.toml

# Devnet (implicit)
chain_type  = "real"
gateway_uri = "https://devnet-gateway.multiversx.com"

# Chain-simulator local
# chain_type  = "simulator"
# gateway_uri = "http://localhost:8085"
```

### 4.5 Verificare pre-deploy

```bash
# Testează conexiunea la Devnet
curl -sf https://devnet-gateway.multiversx.com/about

# Verifică soldul wallet-ului operator (necesari EGLD pentru gas)
# Faucet Devnet: https://devnet-wallet.multiversx.com/faucet
```

---

## 5. Demo interactiv

### 5.1 Script demo (terminal)

```bash
# Acordă permisiuni de execuție
chmod +x demo/demo.sh

# Setează cheia
export EDUCHAIN_PEM="$(cat ~/wallets/devnet_operator.pem)"

# Rulează demo interactiv (cu pauze)
./demo/demo.sh

# Rulează un pas specific
./demo/demo.sh 3   # Pasul 3 — Emitere certificat

# Rulează toți pașii automat (mod CI)
AUTO=1 ./demo/demo.sh a

# Sari deploy-ul (contracte deja existente)
SKIP_DEPLOY=1 ./demo/demo.sh a
```

**Cei 7 pași ai demo-ului:**

| Pas | Titlu | Contract |
|---|---|---|
| 1 | Deploy toate contractele pe Devnet | Toate 4 |
| 2 | Emitere token $ECT și inițializare trezorerie | `ect-token` |
| 3 | Emitere certificat SBT pentru student | `edu-chain-nft` |
| 4 | Distribuire recompense $ECT | `ect-token` |
| 5 | Sesiune mentoring cu escrow | `ect-escrow` |
| 6 | Ancorare hash document pe blockchain | `ect-anchor` |
| 7 | Revocare certificat (soft revoke) | `edu-chain-nft` |

### 5.2 Interfață grafică (GUI) pentru stakeholderi

Pentru prezentări, deschide `demo/demo-gui.html` în orice browser:

```bash
open demo/demo-gui.html
# sau
xdg-open demo/demo-gui.html
```

GUI-ul oferă:
- **Tab Contracte** — Carduri cu adresele deployate și linkuri directe la Explorer
- **Tab Demo Pași** — Cei 7 pași expandabili cu flux de execuție și simulare terminal
- **Tab Arhitectură** — Diagrama vizuală a sistemului
- **Tab Setup** — Instrucțiuni de configurare și linkuri utile

---

## 6. Referință CLI

Binarul `rust-interact` expune subcomenzile: `deploy`, `nft`, `token`, `escrow`, `anchor`, `e2e`.
Configurația se citește din `interactor/config.toml`; adresele deployate sunt persistate în `interactor/state.toml`.

### Deploy

```bash
# Deploy un singur contract
rust-interact deploy nft
rust-interact deploy token
rust-interact deploy escrow
rust-interact deploy anchor

# Deploy toate 4 contracte în ordine
rust-interact deploy all
```

### Comenzi NFT (`edu-chain-nft`)

```bash
# Inițializare colecție (prima dată)
rust-interact nft issue-token --name EduCertificate --ticker EDUCERT
rust-interact nft set-local-roles
rust-interact nft add-issuer --addr erd1...

# Gestionare emitere
rust-interact nft issue    --cert-file fixtures/cert1.json
rust-interact nft revoke   --nonce 1 --reason "date eronate"
rust-interact nft verify   --nonce 1

# Gestionare issueri
rust-interact nft add-issuer    --addr erd1...
rust-interact nft remove-issuer --addr erd1...
```

### Comenzi Token (`ect-token`)

```bash
# Emitere token (o singură dată)
rust-interact token issue --name EduChainToken --ticker ECT --supply 10000000

# Trezorerie
rust-interact token mint-rewards       --amount 1000000
rust-interact token treasury-balance

# Recompense
rust-interact token add-distributor    --addr erd1...
rust-interact token add-reward         --to erd1... --amount 500 --reason "absolvire"
rust-interact token pending-balance    --address erd1...
rust-interact token claim
```

### Comenzi Escrow (`ect-escrow`)

```bash
rust-interact escrow set-token-id
rust-interact escrow create       --mentor erd1... --amount 200 --deadline-hours 72
rust-interact escrow get-session  --id 0
rust-interact escrow confirm      --id 0
rust-interact escrow dispute      --id 0
rust-interact escrow resolve      --id 0 --winner mentor
rust-interact escrow expire       --id 0
```

### Comenzi Anchor (`ect-anchor`)

```bash
# Ancorare fișier local
rust-interact anchor put    --file ./docs/EduChain_TRL4_E2_Validation_Report.md

# Ancorare hash direct
rust-interact anchor put    --hex <64-char-hex> --uri https://example.com/doc

# Verificare
rust-interact anchor verify --hex <64-char-hex>

# Revocare
rust-interact anchor revoke --hex <64-char-hex> --reason "document retras"
```

### Scenariul E2E complet

```bash
rust-interact e2e --scenario default
```

Rulează fluxul complet: deploy all → emitere token → mint rewards → emitere certificat → ancorare hash → creare escrow → confirmare sesiune.

---

## 7. Schema metadate off-chain

Metadatele complete ale certificatelor sunt stocate off-chain (IPFS / backend) în format **JSON-LD** conform specificațiilor **Open Badge 3.0** și **European Learning Model (ELM)**. Hash-ul SHA-256 al documentului canonic (chei sortate lexicografic) este stocat on-chain în câmpul `content_hash`.

### 7.1 Stratificarea câmpurilor (P0 / P1 / P2)

| Câmp JSON-LD | Prioritate | On-Chain | Descriere |
|---|---|---|---|
| `@context` | P0 | ✗ | URIs JSON-LD context |
| `id` | P0 | via nonce | `urn:educhain:cert:<nonce>` |
| `issuer.id` | P0 | ✗ | DID instituție |
| `issuer.name` | P0 | `institution_name` | Numele instituției |
| `validFrom` | P0 | `creation_timestamp` | ISO 8601 |
| `validUntil` | P0 | `expiration_timestamp` | ISO 8601 sau null |
| `credentialSubject.id` | P0 | `student_address` | Wallet bech32 |
| `credentialSubject.achievement.id` | P0 | `course_id` | ID curs |
| `credentialSubject.achievement.name` | P0 | `program_name` | Denumire program |
| `credentialSubject.achievement.type` | P0 | `achievement_type` | Tip realizare |
| `instructor.name` | P0 | `instructor_name` | Instructor |
| `language` | P0 | `language` | ISO 639-1 |
| `credentialSubject.result.value` | P1 | `grade` | Notă numerică |
| `credentialSubject.result.ectsCredits` | P1 | `ects_credits` | Credite ECTS |
| `credentialSubject.achievement.alignment` | P1 | `eqf_level` | Nivel EQF (1–8) |
| `credentialSubject.achievement.criteria` | P2 | ✗ | URL criterii |
| `credentialSubject.achievement.skills` | P2 | ✗ | Lista ESCO |
| `credentialSubject.evidence` | P2 | ✗ | URL-uri dovezi |
| `content_hash` | P0 | `content_hash` | SHA-256 canonic |

Schema completă: [`docs/schemas/educhain_metadata_v1.0.jsonld`](docs/schemas/educhain_metadata_v1.0.jsonld)

### 7.2 Exemplu fixture certificat (`fixtures/cert1.json`)

```json
{
  "student_address": "erd1...",
  "course": "Blockchain Engineering 101",
  "institution": "Universitatea Politehnica București",
  "grade": "10",
  "ects_credits": 6,
  "eqf_level": 6,
  "language": "ro",
  "issued_at": "2025-05-01",
  "metadata_offchain_path": "fixtures/cert1.metadata.jsonld"
}
```

### 7.3 Calculul content_hash

```
content_hash = SHA-256(canonical_json_ld)

unde canonical_json_ld = JSON cu cheile sortate lexicografic, fără spații
```

Calculul este efectuat de interactor (crate `sha2`) înainte de a trimite tranzacția. Contractul stochează și verifică hash-ul, dar nu îl recalculează.

---

## 8. Testare

### 8.1 Rulare teste

```bash
# Toate testele din workspace (unit + scenario runner MultiversX)
cargo test --workspace

# Un singur contract
cargo test -p edu-chain-nft

# Cu output detaliat
cargo test --workspace -- --nocapture
```

### 8.2 Acoperire scenarii

Fișierele de scenarii se află în `contracts/<contract>/scenarios/` și sunt executate de `multiversx-sc-scenario` prin funcții `#[test]` din `tests/`.

**`edu-chain-nft` — 5 scenarii:**

| Fișier | Verificare |
|---|---|
| `01_deploy.scen.json` | Deploy, owner setat corect |
| `02_issue_token.scen.json` | Creare colecție SBT, token ID stocat |
| `03_issue_cert.scen.json` | Emitere certificat, `content_hash` corect on-chain |
| `04_revoke_cert.scen.json` | Revocare by issuer → status `Revoked`; non-issuer → eroare |
| `05_batch_issue.scen.json` | Batch 3 certificate, toate atributele corecte |

**`ect-token` — 3 scenarii:**

| Fișier | Verificare |
|---|---|
| `01_deploy_issue.scen.json` | Deploy, emitere $ECT, token ID stocat |
| `02_mint_rewards.scen.json` | Mint în trezorerie, sold verificat |
| `03_claim_rewards.scen.json` | Adaugă reward, claim, transfer verificat |

**`ect-escrow` — 3 scenarii:**

| Fișier | Verificare |
|---|---|
| `01_create_session.scen.json` | Student creează sesiune, fonduri blocate |
| `02_confirm_completion.scen.json` | Student confirmă, fonduri la mentor |
| `03_expire_session.scen.json` | Deadline depășit, student refundat |

**`ect-anchor` — 3 scenarii:**

| Fișier | Verificare |
|---|---|
| `01_anchor_hash.scen.json` | Ancorare hash 32 bytes, stocat corect |
| `02_duplicate_rejected.scen.json` | Al doilea anchor cu același hash → eroare |
| `03_revoke_anchor.scen.json` | Autor revocă, `revoked == true` verificat |

### 8.3 Analiză statică

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

---

## 9. CI/CD

### `ci.yml` — La fiecare PR și push pe `main`/`develop`

```
1. cargo fmt --all -- --check
2. cargo clippy --workspace --all-targets -- -D warnings
3. cargo build --workspace
4. cargo test --workspace
5. Build WASM pentru toate 4 contracte → artifact wasm-outputs
```

### `devnet.yml` — Nightly (02:00 UTC) sau manual

```
1. Build WASM pentru toate 4 contracte
2. rust-interact deploy all → Devnet
3. rust-interact e2e --scenario default
4. Upload state.toml ca artifact devnet-audit-<run>
```

**Declanșare manuală:**
GitHub → Actions → `Nightly Devnet E2E` → `Run workflow`

**Secret necesar:** `EDUCHAIN_PEM` — conținutul fișierului PEM al wallet-ului operator.

---

## 10. Sandbox local

Sandbox-ul oferă un mediu de dezvoltare complet local bazat pe **MultiversX chain-simulator**.

```bash
cd sandbox

# Pornire servicii (chain-simulator + mock-backend)
docker compose up -d

# Deploy toate contractele pe chain-simulator
bash deploy_all.sh

# Oprire și resetare completă (șterge starea chain-ului)
docker compose down -v
```

**Servicii:**

| Serviciu | Port | Descriere |
|---|---|---|
| `chain-simulator` | 8085 | MultiversX chain-simulator |
| `interactor` | — | Rulează `deploy all` și iese (one-shot) |
| `mock-backend` | 3001 | Server Node.js stub pentru metadata off-chain |

**Configurare pentru sandbox:**

```toml
# interactor/config.toml — dezcommentează pentru sandbox
chain_type  = "simulator"
gateway_uri = "http://localhost:8085"
```

---

## 11. Securitate

### 11.1 Controale de acces implementate

| Control | Implementare | Nivel risc |
|---|---|---|
| `issueCertificate` — issueri autorizați | `require_authorized_issuer()` guard | Scăzut |
| `revokeCertificate` — issueri autorizați | Același guard | Scăzut |
| `mintRewards` — doar owner | `#[only_owner]` | Scăzut |
| `addReward` — distribuitori autorizați | `require_authorized_distributor()` | Scăzut |
| `resolveDispute` — doar owner | `#[only_owner]` | Scăzut |

### 11.2 Limitări cunoscute (TRL-4)

1. **Fără mecanism de pauză** — În caz de urgență, owner-ul nu poate opri emisiile. Mitigare: elimină toți issuerii din whitelist ca freeze temporar.

2. **Soft revocation only** — NFT-urile revocate rămân în wallet-ul studentului. Aplicația trebuie să verifice statusul înainte de a afișa un certificat ca valid.

3. **Owner unic** — Operațiile critice (issueToken, mintRewards, resolveDispute) nu sunt protejate de multi-sig. TRL-5 va adăuga un contract controller multi-sig.

4. **Hash calculat off-chain** — SHA-256 este calculat de interactor și trimis contractului care îl acceptă ca atare. TRL-5 va evalua utilizarea funcțiilor crypto builtin MultiversX sau dovezi ZK.

5. **Fără validare URL off-chain** — Contractul nu verifică că `verification_url` din atribute returnează JSON-LD-ul corect la momentul emiterii.

### 11.3 Recomandare audit

Înainte de TRL-5 / pre-producție:
- Audit extern de contract smart de la o firmă specializată MultiversX
- Verificare formală a logicii de eliberare fonduri escrow
- Stress testing limite gas pentru operații batch

### 11.4 GDPR

- **Nicio dată personală stocată on-chain.** Câmpul `name` (P1) este opțional — se recomandă identificatori pseudonimi.
- Endpoint-ul `registerUser` din TRL-3 nu a fost portat (GDPR Art. 9.2 — stocarea on-chain a numelor/email-urilor este risc ridicat pentru minimizarea datelor).
- Metadatele JSON-LD off-chain care conțin date personale trebuie stocate cu controale de acces adecvate și politici de retenție.

---

## 12. Conformitate și standarde

| Standard | Utilizare | Conformanță |
|---|---|---|
| Open Badge 3.0 (IMS Global) | Structura credential off-chain | Parțială (câmpuri core) |
| European Learning Model (ELM) | Aliniere EQF, credite ECTS | Parțială |
| W3C Verifiable Credentials v2 | `@context`, `type`, `issuer`, `validFrom` | Parțială |
| ESCO Skills Taxonomy | ID-uri skills în JSON-LD | Referință |
| EQF Level codes | Câmpul `eqf_level` | Completă |
| ISO 639-1 | Câmpul `language` | Completă |

Conformanța completă OB3/ELM necesită o semnătură W3C Data Integrity (Ed25519Signature2020) peste documentul JSON-LD — țintă TRL-5.

---

## 13. Performanță și costuri gas

| Operație | Gas estimat | Cost EGLD (Devnet) |
|---|---|---|
| `issueCertificate` (single) | ~15M | ~0.0015 EGLD |
| `issueCertificateBatch` (50) | ~700M | ~0.07 EGLD |
| `anchorContent` | ~5M | ~0.0005 EGLD |
| `createSession` (escrow) | ~8M | ~0.0008 EGLD |
| `confirmCompletion` (escrow) | ~6M | ~0.0006 EGLD |
| `claimRewards` (token) | ~6M | ~0.0006 EGLD |

Estimările se bazează pe observații chain-simulator locale și variază cu dimensiunea payload-ului. Optimizarea gas este planificată pentru TRL-5 (Mainnet).

---

## 14. Roadmap TRL

```
TRL-3  ──► TRL-4 E1  ──► TRL-4 E2 (ACUM)  ──► TRL-5
  │            │               │                  │
  │            │               │                  │
  ▼            ▼               ▼                  ▼
PoC         Arhitectură    4 contracte        Pre-producție
single      multi-          Devnet live        Testnet
contract    contract        CLI interactor     Audit extern
            schițată        CI/CD complet      Multi-sig
                            Demo GUI           W3C Data Integrity
                                               IPFS metadata
                                               Portal verificare
                                               SDK mobil
```

### TRL-5 — Next steps

1. **W3C Data Integrity signatures** peste JSON-LD (Ed25519Signature2020)
2. **Multi-sig owner wallet** pentru operații critice
3. **Hashing on-chain** prin MultiversX crypto builtins
4. **Integrare IPFS** pentru storage decentralizat metadate
5. **Deploy Testnet** și portal public de verificare certificate
6. **Audit extern de securitate** smart contracts
7. **SDK mobil** pentru afișare și verificare certificate
8. **Contract de guvernanță** pentru managementul whitelist-ului de issueri
9. **Suite completă OB3 + W3C VC conformance tests**

---

## 15. Documentație suplimentară

| Document | Descriere |
|---|---|
| [`docs/RUNBOOK_OPS.md`](docs/RUNBOOK_OPS.md) | Runbook operațional: deployment, key rotation, incident response, rollback |
| [`docs/EduChain_TRL4_E2_Validation_Report.md`](docs/EduChain_TRL4_E2_Validation_Report.md) | Raport validare TRL-4 E2: sprint delivery, test results, KPI, security assessment |
| [`docs/schemas/educhain_metadata_v1.0.jsonld`](docs/schemas/educhain_metadata_v1.0.jsonld) | Schema JSON-LD metadate off-chain v1.0 |
| [`demo/demo.sh`](demo/demo.sh) | Script demo interactiv (7 pași, suport AUTO/SKIP_DEPLOY) |
| [`demo/demo-gui.html`](demo/demo-gui.html) | Interfață grafică demo pentru stakeholderi (browser) |
| [`interactor/config.toml`](interactor/config.toml) | Configurație gateway, key source |
| [`interactor/state.toml`](interactor/state.toml) | Adrese contracte deployate |

---

## 16. Referințe

- [MultiversX SC Framework 0.65](https://docs.multiversx.com/developers/developer-reference/rust-smart-contract-framework/)
- [MultiversX ESDT Standard](https://docs.multiversx.com/tokens/esdt-tokens)
- [Open Badge 3.0 Specification](https://www.imsglobal.org/spec/ob/v3p0/)
- [European Learning Model (ELM)](https://joinup.ec.europa.eu/collection/semantic-interoperability-community-semic/solution/european-learning-model)
- [W3C Verifiable Credentials v2](https://www.w3.org/TR/vc-data-model-2.0/)
- [EQF — European Qualifications Framework](https://europa.eu/europass/en/description-eight-eqf-levels)
- [ESCO Skills Taxonomy](https://esco.ec.europa.eu/en/about-esco/what-esco)
- [MultiversX Devnet Explorer](https://devnet-explorer.multiversx.com)
- [MultiversX Devnet Faucet](https://devnet-wallet.multiversx.com/faucet)

---

*EduChain PoC — TRL-4 Etapa 2 · Autor: Costin Carabas · 2026*
