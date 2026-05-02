#!/bin/bash
# Montana node — установка одной командой на Linux VPS.
#
# Использование (одна строка в терминале VPS):
#   curl -sSL https://raw.githubusercontent.com/efir369999/Montana/main/Код/scripts/install-vps.sh | sudo bash
#
# Либо локально после git clone:
#   sudo bash scripts/install-vps.sh
#
# Что делает:
#   1. Проверяет OS (Ubuntu/Debian/Fedora/RHEL/Alpine)
#   2. Устанавливает system deps (build-essential, clang, git, perl)
#   3. Ставит Rust toolchain через rustup (если нет)
#   4. Клонирует/обновляет репозиторий в /opt/montana
#   5. Собирает release бинарь
#   6. Создаёт системного пользователя montana и /var/lib/montana
#   7. Генерирует identity (24-словная мнемоника выводится — ЗАПИШИТЕ!)
#   8. Устанавливает systemd unit с hardening
#   9. Запускает узел и включает автозапуск при старте VPS

set -euo pipefail

REPO_URL="${MONTANA_REPO_URL:-https://github.com/efir369999/Montana.git}"
REPO_BRANCH="${MONTANA_REPO_BRANCH:-main}"
INSTALL_DIR="/opt/montana"
DATA_DIR="/var/lib/montana"
BIN_DST="/usr/local/bin/montana-node"
USER_NAME="montana"
SERVICE_FILE="/etc/systemd/system/montana-node.service"

log() { printf '\033[1;32m[install-vps]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[install-vps]\033[0m %s\n' "$*" >&2; }
die() { printf '\033[1;31m[install-vps] ОШИБКА:\033[0m %s\n' "$*" >&2; exit 1; }

# --- Шаг 1: проверка root ---
if [ "$(id -u)" != "0" ]; then
  die "требуется sudo/root. Запустите: curl -sSL <URL> | sudo bash"
fi

# --- Шаг 2: detect OS ---
if [ ! -f /etc/os-release ]; then
  die "не могу определить OS — отсутствует /etc/os-release"
fi
. /etc/os-release
OS_ID="${ID:-unknown}"
log "обнаружен OS: ${PRETTY_NAME:-$OS_ID}"

# --- Шаг 3: install system deps ---
log "устанавливаю system dependencies..."
case "$OS_ID" in
  ubuntu|debian)
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq
    apt-get install -y -qq build-essential clang pkg-config git curl perl ca-certificates >/dev/null
    ;;
  fedora|rhel|centos|rocky|almalinux)
    dnf install -y -q gcc gcc-c++ clang pkgconf-pkg-config git curl perl ca-certificates make >/dev/null
    ;;
  alpine)
    apk add --no-cache build-base clang pkgconfig git curl perl linux-headers ca-certificates >/dev/null
    ;;
  *)
    die "неподдерживаемый OS: $OS_ID. Поддерживаются: ubuntu, debian, fedora, rhel, centos, rocky, almalinux, alpine"
    ;;
esac

# --- Шаг 4: install Rust toolchain ---
if ! command -v cargo >/dev/null 2>&1; then
  log "устанавливаю Rust toolchain (rustup)..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain stable --profile minimal --no-modify-path
  export PATH="/root/.cargo/bin:$PATH"
else
  log "Rust toolchain уже установлен: $(cargo --version)"
fi
export PATH="${HOME:-/root}/.cargo/bin:/root/.cargo/bin:$PATH"

# --- Шаг 5: clone/update repo ---
if [ -d "$INSTALL_DIR/.git" ]; then
  log "обновляю репозиторий $INSTALL_DIR..."
  cd "$INSTALL_DIR"
  git fetch origin "$REPO_BRANCH"
  git reset --hard "origin/$REPO_BRANCH"
else
  log "клонирую $REPO_URL (branch $REPO_BRANCH) → $INSTALL_DIR..."
  rm -rf "$INSTALL_DIR"
  git clone --branch "$REPO_BRANCH" --single-branch "$REPO_URL" "$INSTALL_DIR"
fi

# --- Шаг 6: build бинарь ---
SOURCE_DIR="$INSTALL_DIR/Код"
if [ ! -d "$SOURCE_DIR" ]; then
  die "директория '$SOURCE_DIR' не найдена в репозитории. Возможно структура изменилась — проверьте путь к montana-node."
fi
cd "$SOURCE_DIR"
log "собираю montana-node release (это занимает 5-15 минут на первом запуске)..."
cargo build --release -p montana-node 2>&1 | tail -5

# --- Шаг 7: install бинарь ---
install -m 0755 target/release/montana-node "$BIN_DST"
log "бинарь установлен: $BIN_DST"

# --- Шаг 8: create user + data dir ---
if ! id "$USER_NAME" >/dev/null 2>&1; then
  log "создаю системного пользователя $USER_NAME..."
  useradd -r -s /usr/sbin/nologin -d "$DATA_DIR" -M "$USER_NAME" 2>/dev/null \
    || useradd -r -s /bin/false -d "$DATA_DIR" "$USER_NAME"
fi
mkdir -p "$DATA_DIR"
chown -R "$USER_NAME:$USER_NAME" "$DATA_DIR"
chmod 0750 "$DATA_DIR"

# --- Шаг 9: init identity (если нет) ---
if [ ! -f "$DATA_DIR/identity.bin" ]; then
  log "генерирую identity (24-словная мнемоника)..."
  echo
  echo "================================================================"
  echo "  ВНИМАНИЕ: ниже выведутся 24 слова мнемоники."
  echo "  ЗАПИШИТЕ их в надёжное место — это backup всего."
  echo "  Потеряете → потеряете доступ к узлу и всем заработанным Ɉ."
  echo "================================================================"
  echo
  sudo -u "$USER_NAME" "$BIN_DST" init --data-dir "$DATA_DIR"
  echo
else
  log "identity уже существует ($DATA_DIR/identity.bin) — пропускаю генерацию"
fi

# --- Шаг 10: install systemd unit ---
log "устанавливаю systemd unit $SERVICE_FILE..."
cat > "$SERVICE_FILE" <<UNIT
[Unit]
Description=Montana Local Node (single-node, Proof-of-Time)
Documentation=https://github.com/efir369999/Montana
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=$USER_NAME
Group=$USER_NAME
ExecStart=$BIN_DST start --data-dir $DATA_DIR
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

# Hardening (per systemd security best-practice)
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=$DATA_DIR
ProtectKernelTunables=yes
ProtectKernelModules=yes
ProtectControlGroups=yes
RestrictRealtime=yes
RestrictSUIDSGID=yes
LockPersonality=yes
MemoryDenyWriteExecute=no
SystemCallArchitectures=native

# Resource limits — узел single-thread, 1 ядро достаточно
CPUQuota=110%
LimitNOFILE=4096

[Install]
WantedBy=multi-user.target
UNIT

# --- Шаг 11: enable & start ---
systemctl daemon-reload
systemctl enable montana-node.service >/dev/null 2>&1
systemctl restart montana-node.service

# --- Финальный отчёт ---
sleep 2
log ""
log "================================================================"
log "  УСТАНОВКА ЗАВЕРШЕНА"
log "================================================================"
log ""
log "Бинарь:        $BIN_DST"
log "Данные:        $DATA_DIR"
log "Пользователь:  $USER_NAME"
log "Service:       montana-node.service"
log ""
log "--- статус узла ---"
systemctl --no-pager status montana-node.service | head -10 || true
log ""
log "Полезные команды:"
log "  systemctl status montana-node       # текущий статус"
log "  journalctl -u montana-node -f       # логи в реальном времени"
log "  systemctl stop montana-node         # остановить узел"
log "  systemctl restart montana-node      # перезапустить"
log "  $BIN_DST status --data-dir $DATA_DIR    # phase + balance"
log ""
log "Жизненный цикл узла:"
log "  Phase 1: Bootstrap → CandidateVdf  (~10-14 часов VDF до vdf_chain_length ≥ τ₂)"
log "  Phase 2: CandidateVdf → Registered (NodeRegistration через canonical apply_*)"
log "  Phase 3: Registered → Active       (selection event на следующем W % 336 == 0)"
log "  Phase 4: Active                    (эмиссия 13 Ɉ per окно через apply_proposal)"
log ""
log "Это spec-compliant Sybil-защита Montana — нельзя обойти быстрее."
log "Узел переживает рестарты VPS (systemd Restart=on-failure) и продолжает с того окна где был."
