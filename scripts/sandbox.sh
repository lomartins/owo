#!/usr/bin/env bash
# Sandbox server — fully isolated from any production owo instance.
# Uses its own DB, uploads dir, and port so it never touches owo.db or :8080.
#
# Usage:
#   ./scripts/sandbox.sh build   # build backend (release) + web SPA
#   ./scripts/sandbox.sh run     # run the sandbox server (foreground)
#   ./scripts/sandbox.sh reset   # delete the sandbox DB
#   ./scripts/sandbox.sh seed    # register a demo user, print its token
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SANDBOX_DIR="$REPO_DIR/.sandbox"
DB_FILE="$SANDBOX_DIR/owo-sandbox.db"
UPLOADS_DIR="$SANDBOX_DIR/uploads"
BIND_ADDR="${SANDBOX_BIND:-127.0.0.1:8091}"
BIN="$REPO_DIR/backend/target/release/owo"

mkdir -p "$SANDBOX_DIR" "$UPLOADS_DIR"

build() {
  echo "==> building backend (release)"
  (cd "$REPO_DIR/backend" && cargo build --release)
  echo "==> building web SPA"
  if [[ ! -d "$REPO_DIR/web/node_modules" ]]; then (cd "$REPO_DIR/web" && npm install); fi
  (cd "$REPO_DIR/web" && npm run build)
}

run() {
  [[ -x "$BIN" ]] || { echo "binary missing — run: $0 build" >&2; exit 1; }
  echo "==> sandbox listening on http://$BIND_ADDR  (db: $DB_FILE)"
  DATABASE_URL="sqlite://$DB_FILE?mode=rwc" \
  BIND_ADDR="$BIND_ADDR" \
  OWO_WEB_DIST="$REPO_DIR/web/dist" \
  OWO_UPLOADS_DIR="$UPLOADS_DIR" \
  exec "$BIN"
}

reset() {
  rm -f "$DB_FILE" "$DB_FILE-wal" "$DB_FILE-shm"
  echo "==> sandbox DB reset"
}

seed() {
  local base="http://$BIND_ADDR/api/v1"
  curl -s -X POST "$base/auth/register" \
    -H 'content-type: application/json' \
    -d '{"email":"demo@owo.test","password":"demo-pass-1234","display_name":"Luisa","default_currency":"BRL","locale":"pt-BR","device_id":"sandbox","device_name":"sandbox"}' \
    | grep -o '"token":"[^"]*"' || echo "register failed (server running?)"
}

case "${1:-run}" in
  build) build ;;
  run)   run ;;
  reset) reset ;;
  seed)  seed ;;
  *) echo "usage: $0 {build|run|reset|seed}" >&2; exit 1 ;;
esac
