#!/usr/bin/env bash
# =============================================================================
#  EduChain PoC — Demo interactiv (Devnet MultiversX)
#  TRL-4 Etapa 2
#
#  Utilizare:
#    chmod +x demo/demo.sh
#    export EDUCHAIN_PEM="$(cat ~/wallets/devnet_operator.pem)"
#    ./demo/demo.sh
#
#  Opțiuni:
#    AUTO=1 ./demo/demo.sh    # rulează fără pauze (mod CI / non-interactiv)
#    SKIP_DEPLOY=1 ./demo/demo.sh   # sare deploy-ul dacă state.toml e populat
# =============================================================================

set -euo pipefail

# ── Culori și formatare ────────────────────────────────────────────────────
BOLD='\033[1m'
DIM='\033[2m'
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
WHITE='\033[1;37m'
RESET='\033[0m'

# ── Configurație ──────────────────────────────────────────────────────────
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INTERACTOR_DIR="$REPO_ROOT/interactor"
BINARY="$REPO_ROOT/target/release/rust-interact"
AUTO="${AUTO:-0}"
SKIP_DEPLOY="${SKIP_DEPLOY:-0}"

DEVNET_EXPLORER="https://devnet-explorer.multiversx.com"

# ── Funcții helper ─────────────────────────────────────────────────────────

banner() {
  echo ""
  echo -e "${CYAN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
  echo -e "${CYAN}${BOLD}  $1${RESET}"
  echo -e "${CYAN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
  echo ""
}

step() {
  local num="$1"
  local title="$2"
  echo ""
  echo -e "${YELLOW}${BOLD}▶ Pasul $num — $title${RESET}"
  echo -e "${DIM}$(printf '─%.0s' {1..60})${RESET}"
}

info() {
  echo -e "${BLUE}ℹ  $*${RESET}"
}

success() {
  echo -e "${GREEN}✔  $*${RESET}"
}

warn() {
  echo -e "${YELLOW}⚠  $*${RESET}"
}

fail() {
  echo -e "${RED}✖  $*${RESET}"
  exit 1
}

run_cmd() {
  # Afișează comanda, o execută și marchează succesul
  echo -e "${DIM}\$ $*${RESET}"
  eval "$@"
  local status=$?
  if [[ $status -eq 0 ]]; then
    success "Comandă executată cu succes (exit 0)"
  else
    fail "Comandă eșuată cu exit $status"
  fi
  return $status
}

pause() {
  if [[ "$AUTO" == "1" ]]; then
    sleep 1
    return
  fi
  echo ""
  echo -e "${MAGENTA}${BOLD}  [ Apasă ENTER pentru a continua... ]${RESET}"
  read -r
}

explorer_link() {
  local type="$1"   # accounts / transactions
  local addr="$2"
  echo -e "${DIM}  🔗 Explorer: ${DEVNET_EXPLORER}/${type}/${addr}${RESET}"
}

# ── Verificare pre-demo ────────────────────────────────────────────────────

preflight_check() {
  banner "🔍 Verificare pre-demo"

  info "Directorul proiectului: $REPO_ROOT"
  info "Binar interactor: $BINARY"

  # Cheie PEM
  if [[ -z "${EDUCHAIN_PEM:-}" ]]; then
    fail "Variabila EDUCHAIN_PEM nu este setată!\n   Rulează: export EDUCHAIN_PEM=\"\$(cat ~/wallets/devnet_operator.pem)\""
  fi
  success "EDUCHAIN_PEM setat (${#EDUCHAIN_PEM} caractere)"

  # Binar compilat
  if [[ ! -f "$BINARY" ]]; then
    warn "Binarul rust-interact nu există. Compilez acum..."
    (cd "$REPO_ROOT" && cargo build --release -p rust-interact 2>&1)
    success "Build finalizat"
  else
    success "Binar găsit: $BINARY"
  fi

  # Conexiune Devnet
  info "Testez conexiunea la Devnet..."
  if curl -sf "https://devnet-gateway.multiversx.com/about" > /dev/null 2>&1; then
    success "Devnet accesibil"
  else
    fail "Nu pot ajunge la Devnet gateway. Verifică conexiunea la internet."
  fi

  echo ""
  success "Pre-flight OK — gata de demo!"
}

# ── Afișare intro ──────────────────────────────────────────────────────────

show_intro() {
  clear
  echo ""
  echo -e "${BOLD}${WHITE}"
  echo "  ███████╗██████╗ ██╗   ██╗ ██████╗██╗  ██╗ █████╗ ██╗███╗   ██╗"
  echo "  ██╔════╝██╔══██╗██║   ██║██╔════╝██║  ██║██╔══██╗██║████╗  ██║"
  echo "  █████╗  ██║  ██║██║   ██║██║     ███████║███████║██║██╔██╗ ██║"
  echo "  ██╔══╝  ██║  ██║██║   ██║██║     ██╔══██║██╔══██║██║██║╚██╗██║"
  echo "  ███████╗██████╔╝╚██████╔╝╚██████╗██║  ██║██║  ██║██║██║ ╚████║"
  echo "  ╚══════╝╚═════╝  ╚═════╝  ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝"
  echo -e "${RESET}"
  echo -e "${CYAN}${BOLD}        Blockchain-powered Educational Certificate System${RESET}"
  echo -e "${DIM}        TRL-4 Etapa 2 — Demo pe Devnet MultiversX${RESET}"
  echo ""
  echo -e "${WHITE}  Contracte demonstrate:${RESET}"
  echo -e "  ${GREEN}●${RESET} edu-chain-nft  — Emitere certificate SBT"
  echo -e "  ${GREEN}●${RESET} ect-token       — Token de recompensă \$ECT"
  echo -e "  ${GREEN}●${RESET} ect-escrow      — Sesiuni de mentoring cu escrow"
  echo -e "  ${GREEN}●${RESET} ect-anchor      — Ancorare hash-uri de conținut"
  echo ""
  echo -e "${DIM}  Explorer: ${DEVNET_EXPLORER}${RESET}"
  echo ""
  pause
}

# ── Pasul 1: Deploy toate contractele ─────────────────────────────────────

demo_deploy() {
  step "1" "Deploy toate contractele pe Devnet"

  if [[ "$SKIP_DEPLOY" == "1" ]]; then
    warn "SKIP_DEPLOY=1 — sar deploy-ul, folosesc adresele din state.toml"
    cat "$INTERACTOR_DIR/state.toml"
    pause
    return
  fi

  info "Construiesc artefactele WASM pentru toate contractele..."
  for c in contracts/edu-chain-nft contracts/ect-token contracts/ect-escrow contracts/ect-anchor; do
    echo -e "${DIM}  Building $c...${RESET}"
    (cd "$REPO_ROOT/$c/meta" && cargo run -- build --no-wasm-opt 2>&1) || \
    warn "Build $c eșuat — poate există deja WASM-ul compilat"
  done
  success "WASM-uri construite"

  echo ""
  info "Deploy edu-chain-nft (certificatul SBT)..."
  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' deploy nft"
  echo ""

  info "Deploy ect-token (token fungibil \$ECT)..."
  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' deploy token"
  echo ""

  info "Deploy ect-escrow (escrow sesiuni mentoring)..."
  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' deploy escrow"
  echo ""

  info "Deploy ect-anchor (registru hash-uri de conținut)..."
  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' deploy anchor"
  echo ""

  success "Toate 4 contracte au fost deployate!"
  echo ""
  info "Adresele deployate (state.toml):"
  cat "$INTERACTOR_DIR/state.toml"
  pause
}

# ── Pasul 2: Inițializare token $ECT ──────────────────────────────────────

demo_token_init() {
  step "2" "Emitere token fungibil \$ECT și inițializare trezorerie"

  info "Emit token-ul \$ECT (EduChain Token)..."
  info "Aceasta este o operație ESDT system — costă 0.05 EGLD și durează ~5s"
  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' token issue --name EduChainToken --ticker ECT --supply 10000000"

  echo ""
  info "Mint recompense inițiale în trezorerie (1,000,000 ECT)..."
  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' token mint-rewards --amount 1000000"

  echo ""
  info "Verific soldul trezoreriei..."
  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' token treasury-balance"

  success "Token \$ECT emis și trezorerie inițializată!"
  pause
}

# ── Pasul 3: Emitere certificat NFT ───────────────────────────────────────

demo_issue_cert() {
  step "3" "Emitere certificat NFT (SBT) pentru un student"

  info "Fixture utilizat: interactor/fixtures/cert1.json"
  echo ""
  echo -e "${DIM}$(cat "$INTERACTOR_DIR/fixtures/cert1.json")${RESET}"
  echo ""

  info "Fluxul complet de emitere:"
  echo -e "  ${DIM}1. Citire fixture JSON${RESET}"
  echo -e "  ${DIM}2. Citire metadata JSON-LD off-chain${RESET}"
  echo -e "  ${DIM}3. Calcul SHA-256 al JSON-LD canonic (chei sortate)${RESET}"
  echo -e "  ${DIM}4. Construire CertificateAttributes cu content_hash${RESET}"
  echo -e "  ${DIM}5. Trimitere tranzacție issueCertificate pe Devnet${RESET}"
  echo -e "  ${DIM}6. NFT transferat automat în wallet-ul studentului${RESET}"
  echo ""
  pause

  info "Setez colecția NFT și rolurile (se ignoră erorile dacă a fost deja inițializată)..."
  cd "$INTERACTOR_DIR" && "$BINARY" nft issue-token --name EduCert --ticker EDUCERT >/dev/null 2>&1 || true
  cd "$INTERACTOR_DIR" && "$BINARY" nft set-local-roles >/dev/null 2>&1 || true
  
  info "Adaug operatorul curent ca issuer autorizat..."
  OPERATOR_ADDR="erd1c3umcqd2f8fx9q605vtqtm3pu8zawac3l96cmrnc35h7uzgvlmpqm3zsnr"
  cd "$INTERACTOR_DIR" && "$BINARY" nft add-issuer --addr "$OPERATOR_ADDR" >/dev/null 2>&1 || true

  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' nft issue --cert-file fixtures/cert1.json"

  echo ""
  info "Verific atributele certificatului emis (nonce=1)..."
  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' nft verify --nonce 1"

  success "Certificat emis cu succes! SHA-256 al metadatelor stocat on-chain."
  pause
}

demo_issue_cert_2() {
  step "3b" "Emitere al doilea certificat NFT (SBT) pentru un student"

  info "Fixture utilizat: interactor/fixtures/cert2.json"
  echo ""
  echo -e "${DIM}$(cat "$INTERACTOR_DIR/fixtures/cert2.json")${RESET}"
  echo ""

  info "Setez colecția NFT și rolurile (se ignoră erorile dacă a fost deja inițializată)..."
  cd "$INTERACTOR_DIR" && "$BINARY" nft issue-token --name EduCert --ticker EDUCERT >/dev/null 2>&1 || true
  cd "$INTERACTOR_DIR" && "$BINARY" nft set-local-roles >/dev/null 2>&1 || true
  
  info "Adaug operatorul curent ca issuer autorizat..."
  OPERATOR_ADDR="erd1c3umcqd2f8fx9q605vtqtm3pu8zawac3l96cmrnc35h7uzgvlmpqm3zsnr"
  cd "$INTERACTOR_DIR" && "$BINARY" nft add-issuer --addr "$OPERATOR_ADDR" >/dev/null 2>&1 || true

  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' nft issue --cert-file fixtures/cert2.json"

  echo ""
  info "Verific atributele certificatului emis (nonce=2)..."
  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' nft verify --nonce 2"

  success "Al doilea certificat emis cu succes!"
  pause
}

demo_issue_cert_3() {
  step "3c" "Emitere al treilea certificat NFT (SBT) pentru un student"

  info "Fixture utilizat: interactor/fixtures/cert3.json"
  echo ""
  echo -e "${DIM}$(cat "$INTERACTOR_DIR/fixtures/cert3.json")${RESET}"
  echo ""

  info "Setez colecția NFT și rolurile (se ignoră erorile dacă a fost deja inițializată)..."
  cd "$INTERACTOR_DIR" && "$BINARY" nft issue-token --name EduCert --ticker EDUCERT >/dev/null 2>&1 || true
  cd "$INTERACTOR_DIR" && "$BINARY" nft set-local-roles >/dev/null 2>&1 || true
  
  info "Adaug operatorul curent ca issuer autorizat..."
  OPERATOR_ADDR="erd1c3umcqd2f8fx9q605vtqtm3pu8zawac3l96cmrnc35h7uzgvlmpqm3zsnr"
  cd "$INTERACTOR_DIR" && "$BINARY" nft add-issuer --addr "$OPERATOR_ADDR" >/dev/null 2>&1 || true

  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' nft issue --cert-file fixtures/cert3.json"

  echo ""
  info "Verific atributele certificatului emis (nonce=3)..."
  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' nft verify --nonce 3"

  success "Al treilea certificat emis cu succes!"
  pause
}

# ── Pasul 4: Recompense studenți ──────────────────────────────────────────

demo_rewards() {
  step "4" "Distribuire recompense \$ECT către student"

  # Extrag adresa studentului din fixture
  STUDENT_ADDR=$(python3 -c "import json; d=open('$INTERACTOR_DIR/fixtures/cert1.json'); print(json.load(d)['student_address'])" 2>/dev/null || echo "erd1...")

  info "Adresă student: $STUDENT_ADDR"
  echo ""

  info "Adaug operatorul curent ca distribuitor autorizat..."
  OPERATOR_ADDR="erd1c3umcqd2f8fx9q605vtqtm3pu8zawac3l96cmrnc35h7uzgvlmpqm3zsnr"
  cd "$INTERACTOR_DIR" && "$BINARY" token add-distributor --addr "$OPERATOR_ADDR" || true

  info "Adaug recompensă de 500 ECT pentru student (pentru absolvire)..."
  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' token add-reward --to '$STUDENT_ADDR' --amount 500 --reason 'absolvire'"

  echo ""
  info "Verific recompensa pending..."
  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' token pending-balance --address '$STUDENT_ADDR'"

  echo ""
  info "Studentul își revendică recompensa..."
  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' token claim"

  success "Recompensele ECT au fost distribuite și revendicate!"
  pause
}

# ── Pasul 5: Sesiune de mentoring cu escrow ────────────────────────────────

demo_escrow() {
  step "5" "Sesiune de mentoring cu escrow \$ECT"

  # Folosim o adresă EOA (wallet normal) pentru a evita eroarea de 'non payable contract'
  MENTOR_ADDR="${DEMO_MENTOR_ADDR:-erd1rdw0zvq472f6hxpd29d2v85xqrlctl3cgu5z8ke8uashxekmhnfs3xxfj5}"

  info "Scenariu: Studentul plătește 200 ECT pentru o sesiune de mentoring"
  info "Mentor: $MENTOR_ADDR"
  echo ""
  echo -e "  ${DIM}Flux:${RESET}"
  echo -e "  ${DIM}1. Student creează sesiunea (blochează 200 ECT în escrow)${RESET}"
  echo -e "  ${DIM}2. Sesiunea rămâne în starea Open pe durata mentoring-ului${RESET}"
  echo -e "  ${DIM}3. Studentul confirmă finalizarea → fonduri eliberate mentorului${RESET}"
  echo ""
  pause

  info "Setez token ID-ul pentru escrow (ignorat dacă e deja setat)..."
  cd "$INTERACTOR_DIR" && "$BINARY" escrow set-token-id || true

  info "Creez sesiunea de mentoring (deadline: 72 ore)..."
  CREATE_OUTPUT=$(cd "$INTERACTOR_DIR" && "$BINARY" escrow create --mentor "$MENTOR_ADDR" --amount 200 --deadline-hours 72)
  echo "$CREATE_OUTPUT"
  
  # Extract the ID from the output (e.g. "ID=1, mentor=...")
  SESSION_ID=$(echo "$CREATE_OUTPUT" | grep "Session created! ID=" | sed -n 's/.*ID=\([0-9]*\).*/\1/p')
  
  if [ -z "$SESSION_ID" ]; then
    SESSION_ID=0 # fallback
  fi

  echo ""
  info "Verific starea sesiunii (ID=$SESSION_ID)..."
  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' escrow get-session --id $SESSION_ID"

  echo ""
  info "Studentul confirmă că sesiunea a fost finalizată..."
  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' escrow confirm --id $SESSION_ID"

  echo ""
  info "Verific că fondurile au ajuns la mentor..."
  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' escrow get-session --id $SESSION_ID"

  success "Escrow: 200 ECT transferați mentorului după confirmare!"
  pause
}

# ── Pasul 6: Ancorare hash document ───────────────────────────────────────

demo_anchor() {
  step "6" "Ancorare hash document pe blockchain"

  info "Anchorez raportul de validare TRL-4 E2..."
  info "Fișier: docs/EduChain_TRL4_E2_Validation_Report.md"
  echo ""
  echo -e "  ${DIM}Fluxul de ancorare:${RESET}"
  echo -e "  ${DIM}1. Calcul SHA-256 al fișierului local${RESET}"
  echo -e "  ${DIM}2. Trimitere hash (32 bytes) + URI pe blockchain${RESET}"
  echo -e "  ${DIM}3. Hash stocat cu: autor, timestamp, URI, revoked=false${RESET}"
  echo ""
  pause

  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' anchor put --file ../docs/EduChain_TRL4_E2_Validation_Report.md"

  echo ""
  # Calculez hash-ul local pentru verificare
  REPORT_HASH=$(sha256sum "$REPO_ROOT/docs/EduChain_TRL4_E2_Validation_Report.md" | awk '{print $1}')
  info "Hash SHA-256 local: $REPORT_HASH"

  echo ""
  info "Verific anchorul pe blockchain..."
  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' anchor verify --hex '$REPORT_HASH'"

  echo ""
  info "Test duplicat — încerc să anchorez același hash din nou (trebuie să dea eroare)..."
  echo -e "${DIM}\$ cd '$INTERACTOR_DIR' && '$BINARY' anchor put --file ../docs/EduChain_TRL4_E2_Validation_Report.md${RESET}"
  if cd "$INTERACTOR_DIR" && "$BINARY" anchor put --file ../docs/EduChain_TRL4_E2_Validation_Report.md 2>&1; then
    warn "Contractul ar trebui să respingă duplicatele!"
  else
    success "Contractul a respins corect hash-ul duplicat!"
  fi

  pause
}

# ── Pasul 7: Revocare certificat ──────────────────────────────────────────

demo_revoke() {
  step "7" "Revocare certificat (soft revoke)"

  info "Simulez revocarea certificatului cu nonce=1 (de ex. date eronate)..."
  echo ""
  echo -e "  ${DIM}Nota: Soft revoke — NFT rămâne în wallet-ul studentului,${RESET}"
  echo -e "  ${DIM}dar verifyCertificate returnează status=Revoked.${RESET}"
  echo ""
  pause

  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' nft revoke --nonce 1 --reason 'date eronate'"

  echo ""
  info "Verific statusul certificatului revocat..."
  run_cmd "cd '$INTERACTOR_DIR' && '$BINARY' nft verify --nonce 1"

  success "Certificatul a fost revocat. Status on-chain: Revoked."
  pause
}

# ── Rezumat final ──────────────────────────────────────────────────────────

show_summary() {
  banner "🎉 Demo finalizat cu succes!"

  echo -e "  ${WHITE}${BOLD}Ce am demonstrat:${RESET}"
  echo ""
  echo -e "  ${GREEN}✔${RESET} ${BOLD}edu-chain-nft${RESET}  — Emitere certificat SBT cu SHA-256 content_hash on-chain"
  echo -e "  ${GREEN}✔${RESET} ${BOLD}ect-token${RESET}       — Token fungibil \$ECT, trezorerie, recompense, claim"
  echo -e "  ${GREEN}✔${RESET} ${BOLD}ect-escrow${RESET}      — Sesiune mentoring: blocare fonduri → confirmare → transfer"
  echo -e "  ${GREEN}✔${RESET} ${BOLD}ect-anchor${RESET}      — Ancorare SHA-256, verificare, respingere duplicat"
  echo -e "  ${GREEN}✔${RESET} ${BOLD}Revocare${RESET}         — Soft revoke certificat cu status on-chain"
  echo ""
  echo -e "  ${DIM}Adrese contracte deployate:${RESET}"
  grep -v '^#' "$INTERACTOR_DIR/state.toml" | grep -v '^$' | while IFS= read -r line; do
    echo -e "  ${DIM}$line${RESET}"
  done
  echo ""
  echo -e "  ${DIM}Explorer Devnet: ${DEVNET_EXPLORER}${RESET}"
  echo ""
  echo -e "  ${DIM}Documentație: docs/EduChain_TRL4_E2_Validation_Report.md${RESET}"
  echo -e "  ${DIM}Runbook ops:   docs/RUNBOOK_OPS.md${RESET}"
  echo ""
  echo -e "${CYAN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
}

# ── Main ──────────────────────────────────────────────────────────────────

main() {
  # Schimbă directorul curent în rădăcina proiectului
  cd "$REPO_ROOT"

  show_intro
  preflight_check

  if [[ $# -gt 0 ]]; then
    CHOICE="$1"
  else
    echo ""
    echo -e "${WHITE}${BOLD}  Alege pasul pe care dorești să îl execuți:${RESET}"
    echo -e "  ${CYAN}1)${RESET} Deploy toate contractele pe Devnet"
    echo -e "  ${CYAN}2)${RESET} Emitere token \$ECT și inițializare trezorerie"
    echo -e "  ${CYAN}3)${RESET} Emitere certificat SBT pentru student (cert1)"
    echo -e "  ${CYAN}3b)${RESET} Emitere al doilea certificat SBT (cert2)"
    echo -e "  ${CYAN}3c)${RESET} Emitere al treilea certificat SBT (cert3)"
    echo -e "  ${CYAN}4)${RESET} Distribuire recompense \$ECT"
    echo -e "  ${CYAN}5)${RESET} Sesiune de mentoring cu escrow"
    echo -e "  ${CYAN}6)${RESET} Ancorare hash document pe blockchain"
    echo -e "  ${CYAN}7)${RESET} Revocare certificat (soft revoke)"
    echo -e "  ${CYAN}a)${RESET} Toți pașii (secvențial)"
    echo -e "  ${CYAN}q)${RESET} Ieșire"
    echo ""
    echo -ne "${YELLOW}${BOLD}  Opțiunea ta: ${RESET}"
    read -r CHOICE
  fi

  case "$CHOICE" in
    1) demo_deploy ;;
    2) demo_token_init ;;
    3) demo_issue_cert ;;
    3b) demo_issue_cert_2 ;;
    3c) demo_issue_cert_3 ;;
    4) demo_rewards ;;
    5) demo_escrow ;;
    6) demo_anchor ;;
    7) demo_revoke ;;
    a)
      demo_deploy
      demo_token_init
      demo_issue_cert
      demo_rewards
      demo_escrow
      demo_anchor
      demo_revoke
      ;;
    q) exit 0 ;;
    *) echo -e "${RED}Opțiune invalidă!${RESET}"; exit 1 ;;
  esac

  show_summary
}

main "$@"
