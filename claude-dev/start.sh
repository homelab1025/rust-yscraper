#!/usr/bin/env bash
set -euo pipefail

# Postgres is started by entrypoint.sh at container boot; this script
# only owns the backend + frontend, run as the non-root "dev" user.

PIDFILE="/workspace/.dev-pids"
LOGDIR="/workspace/logs"

mkdir -p "$LOGDIR"
rm -f "$PIDFILE"

# ── Config file ──────────────────────────────────────────────────────
# web_server reads config.toml from its CWD; the dev copy lives at
# conf/config.toml and isn't meant to be committed at the repo root.
if [[ ! -f /workspace/config.toml ]]; then
  cp /workspace/conf/config.toml /workspace/config.toml
  echo "✅ Copied conf/config.toml to /workspace/config.toml"
fi

# ── Start Backend ───────────────────────────────────────────────────
echo "🚀 Starting backend on :3000..."
cd /workspace
cargo run -p web_server > "$LOGDIR/backend.log" 2>&1 &
CARGO_PID=$!

BACKEND_PID=""
for i in $(seq 1 20); do
  BACKEND_PID=$(pgrep -P "$CARGO_PID" -n 2>/dev/null || true)
  [[ -n "$BACKEND_PID" ]] && break
  sleep 0.5
done
BACKEND_PID="${BACKEND_PID:-$CARGO_PID}"
echo "$BACKEND_PID" > "$PIDFILE"
echo "   PID: $BACKEND_PID"

# Wait for backend to be ready. A cold `cargo run` compiles the whole
# workspace first, which is much slower than booting an already-built
# binary, so this gives it up to a few minutes.
echo "⏳ Waiting for backend (first build can take a while)..."
for i in $(seq 1 180); do
  if curl -sf "http://localhost:3000/ping?msg=ready" > /dev/null 2>&1; then
    echo "✅ Backend is up"
    break
  fi
  sleep 1
done

# ── Start Frontend ──────────────────────────────────────────────────
echo "🌐 Starting frontend on :5173..."
cd /workspace/webapp
npm run dev -- --host 0.0.0.0 > "$LOGDIR/frontend.log" 2>&1 &
FRONTEND_PID=$!

NODE_PID=""
for i in $(seq 1 20); do
  NODE_PID=$(pgrep -P "$FRONTEND_PID" -n 2>/dev/null || true)
  [[ -n "$NODE_PID" ]] && break
  sleep 0.5
done
NODE_PID="${NODE_PID:-$FRONTEND_PID}"
echo "$NODE_PID" >> "$PIDFILE"
echo "   PID: $NODE_PID"

# ── Summary ─────────────────────────────────────────────────────────
echo ""
echo "═══════════════════════════════════════════"
echo "  🌍  rust-yscraper is running!"
echo "═══════════════════════════════════════════"
echo "  Frontend:  http://localhost:5173"
echo "  Backend:   http://localhost:3000"
echo "  Database:  localhost:5432  (db: yscraper, user: postgres, started at container boot)"
echo "═══════════════════════════════════════════"
echo ""
echo "  Stop:  ./stop.sh"
echo "  Force: ./stop.sh --force"
echo "  Logs:  tail -f $LOGDIR/backend.log"
echo "         tail -f $LOGDIR/frontend.log"
echo ""
