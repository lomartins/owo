#!/usr/bin/env bash
# Install owo as a systemd --user service that starts on boot.
# Idempotent: re-run after `git pull` to rebuild + restart.
#
# Usage: ./scripts/install.sh [--no-linger] [--bind 0.0.0.0:8080]
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVICE_NAME="owo"
UNIT_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
UNIT_FILE="$UNIT_DIR/$SERVICE_NAME.service"

BIND_ADDR="0.0.0.0:8080"
ENABLE_LINGER=1

while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-linger) ENABLE_LINGER=0; shift ;;
    --bind) BIND_ADDR="$2"; shift 2 ;;
    -h|--help)
      sed -n '2,8p' "$0"; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

say() { printf '\033[1;36m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m!!\033[0m %s\n' "$*" >&2; }
die() { printf '\033[1;31mXX\033[0m %s\n' "$*" >&2; exit 1; }

# ---- prereqs ----
command -v cargo >/dev/null || die "cargo not found — install Rust toolchain"
command -v npm   >/dev/null || die "npm not found — install Node"
command -v systemctl >/dev/null || die "systemctl not found — this script targets systemd"

# ---- build backend ----
say "building backend (release)"
(cd "$REPO_DIR/backend" && cargo build --release)
BIN="$REPO_DIR/backend/target/release/owo"
[[ -x "$BIN" ]] || die "binary missing after build: $BIN"

# ---- build web ----
say "building web SPA"
if [[ ! -d "$REPO_DIR/web/node_modules" ]]; then
  (cd "$REPO_DIR/web" && npm ci)
fi
(cd "$REPO_DIR/web" && npm run build)
[[ -f "$REPO_DIR/web/dist/index.html" ]] || die "web/dist missing after build"

# ---- runtime dirs ----
mkdir -p "$REPO_DIR/uploads"

# ---- write unit ----
say "writing unit: $UNIT_FILE"
mkdir -p "$UNIT_DIR"
cat > "$UNIT_FILE" <<EOF
[Unit]
Description=owo (self-hosted personal finance)
After=network.target

[Service]
Type=simple
WorkingDirectory=$REPO_DIR
ExecStart=$BIN
Environment=DATABASE_URL=sqlite://$REPO_DIR/owo.db?mode=rwc
Environment=BIND_ADDR=$BIND_ADDR
Environment=OWO_WEB_DIST=$REPO_DIR/web/dist
Environment=OWO_UPLOADS_DIR=$REPO_DIR/uploads
Restart=on-failure
RestartSec=3

[Install]
WantedBy=default.target
EOF

# ---- enable + start ----
say "reloading systemd --user"
systemctl --user daemon-reload

if systemctl --user is-active --quiet "$SERVICE_NAME"; then
  say "restarting $SERVICE_NAME"
  systemctl --user restart "$SERVICE_NAME"
else
  say "enabling + starting $SERVICE_NAME"
  systemctl --user enable --now "$SERVICE_NAME"
fi

# ---- linger so it survives logout / starts on boot ----
if [[ "$ENABLE_LINGER" -eq 1 ]]; then
  if loginctl show-user "$USER" 2>/dev/null | grep -q 'Linger=yes'; then
    say "linger already enabled for $USER"
  else
    say "enabling linger (sudo required)"
    sudo loginctl enable-linger "$USER"
  fi
fi

# ---- status ----
sleep 1
systemctl --user --no-pager status "$SERVICE_NAME" | head -n 10 || true

echo
say "done. listening on http://$BIND_ADDR"
echo "  logs:    journalctl --user -u $SERVICE_NAME -f"
echo "  restart: systemctl --user restart $SERVICE_NAME"
echo "  stop:    systemctl --user stop $SERVICE_NAME"
echo "  disable: systemctl --user disable --now $SERVICE_NAME"
