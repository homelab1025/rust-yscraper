#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
STATE_DIR="$SCRIPT_DIR/.dev"
ENV_FILE="$STATE_DIR/env"
SERVER_PID_FILE="$STATE_DIR/server.pid"
WEBAPP_PID_FILE="$STATE_DIR/webapp.pid"
SERVER_LOG="$STATE_DIR/server.log"
WEBAPP_LOG="$STATE_DIR/webapp.log"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"

usage() {
  echo "Usage: $(basename "$0") {start|status|stop}"
  echo
  echo "Runs Postgres + migrations, the Rust server and the Vite dev server for"
  echo "this worktree. Ports are derived from the worktree's path, so several"
  echo "worktrees can each run their own copy of this stack at the same time"
  echo "without their ports (or Postgres data) colliding."
  exit 1
}

# Derive a stable per-worktree offset from the worktree's absolute path, so the
# same worktree gets the same ports across restarts. Collisions between two
# worktrees are possible but unlikely (900 possible offsets).
compute_ports() {
  local checksum
  checksum="$(printf '%s' "$SCRIPT_DIR" | cksum | cut -d' ' -f1)"
  OFFSET=$((10#$checksum % 900))
  SERVER_PORT=$((3000 + OFFSET))
  DB_PORT=$((5432 + OFFSET))
  ADMINER_PORT=$((8080 + OFFSET))
  WEB_PORT=$((5173 + OFFSET))
  COMPOSE_PROJECT_NAME="yscraper-$OFFSET"
}

write_env_file() {
  mkdir -p "$STATE_DIR"
  cat >"$ENV_FILE" <<EOF
OFFSET=$OFFSET
SERVER_PORT=$SERVER_PORT
DB_PORT=$DB_PORT
ADMINER_PORT=$ADMINER_PORT
WEB_PORT=$WEB_PORT
COMPOSE_PROJECT_NAME=$COMPOSE_PROJECT_NAME
EOF
}

# Populates the port/project variables from a previous `start`, so `status`
# and `stop` target the same containers/ports without recomputing anything.
load_env_file() {
  if [[ -f "$ENV_FILE" ]]; then
    # shellcheck disable=SC1090
    source "$ENV_FILE"
    return 0
  fi
  return 1
}

compose() {
  COMPOSE_PROJECT_NAME="$COMPOSE_PROJECT_NAME" DB_PORT="$DB_PORT" ADMINER_PORT="$ADMINER_PORT" \
    docker compose -f "$COMPOSE_FILE" "$@"
}

pid_running() {
  local pid_file="$1"
  if [[ -f "$pid_file" ]] && kill -0 "$(cat "$pid_file")" 2>/dev/null; then
    return 0
  fi
  return 1
}

# Stops a backgrounded process along with its direct children (e.g. `cargo
# run`'s compiled binary, or `npm run dev`'s vite process).
stop_pid() {
  local pid_file="$1"
  local label="$2"
  if pid_running "$pid_file"; then
    local pid
    pid="$(cat "$pid_file")"
    echo "Stopping $label (pid $pid)..."
    pkill -TERM -P "$pid" 2>/dev/null || true
    kill -TERM "$pid" 2>/dev/null || true
  fi
  rm -f "$pid_file"
}

print_ports() {
  echo "Worktree: $SCRIPT_DIR"
  echo "Ports — server: $SERVER_PORT  db: $DB_PORT  adminer: $ADMINER_PORT  web: $WEB_PORT"
  echo "Compose project: $COMPOSE_PROJECT_NAME"
}

cmd_start() {
  compute_ports
  write_env_file
  print_ports
  echo

  if [[ ! -f "$SCRIPT_DIR/config.toml" ]]; then
    echo "Copying conf/config.toml -> config.toml"
    cp "$SCRIPT_DIR/conf/config.toml" "$SCRIPT_DIR/config.toml"
  fi

  echo "Starting Postgres..."
  compose up -d --wait postgres

  echo "Applying migrations..."
  compose up --no-deps liquibase

  if pid_running "$SERVER_PID_FILE"; then
    echo "web_server already running (pid $(cat "$SERVER_PID_FILE"))"
  else
    echo "Starting web_server on port $SERVER_PORT (log: $SERVER_LOG)..."
    (
      cd "$SCRIPT_DIR"
      YSCR_PORT="$SERVER_PORT" YSCR_DB_PORT="$DB_PORT" \
        nohup cargo run -p web_server >"$SERVER_LOG" 2>&1 &
      echo $! >"$SERVER_PID_FILE"
    )
  fi

  if pid_running "$WEBAPP_PID_FILE"; then
    echo "webapp already running (pid $(cat "$WEBAPP_PID_FILE"))"
  else
    if [[ ! -d "$SCRIPT_DIR/webapp/node_modules" ]]; then
      echo "Installing webapp dependencies..."
      (cd "$SCRIPT_DIR/webapp" && npm install)
    fi
    echo "Starting webapp on port $WEB_PORT (log: $WEBAPP_LOG)..."
    (
      cd "$SCRIPT_DIR/webapp"
      API_PORT="$SERVER_PORT" WEB_PORT="$WEB_PORT" \
        nohup npm run dev >"$WEBAPP_LOG" 2>&1 &
      echo $! >"$WEBAPP_PID_FILE"
    )
  fi

  echo
  echo "Backend:  http://localhost:$SERVER_PORT (Swagger UI at /swagger-ui)"
  echo "Frontend: http://localhost:$WEB_PORT"
  echo "Adminer:  DB_PORT=$DB_PORT ADMINER_PORT=$ADMINER_PORT COMPOSE_PROJECT_NAME=$COMPOSE_PROJECT_NAME docker compose up -d adminer  (then http://localhost:$ADMINER_PORT)"
  echo "Logs:     $SERVER_LOG , $WEBAPP_LOG"
}

cmd_status() {
  if ! load_env_file; then
    echo "No dev stack has been started for this worktree yet. Run: $(basename "$0") start"
    exit 1
  fi

  print_ports
  echo

  if pid_running "$SERVER_PID_FILE"; then
    echo "web_server: running (pid $(cat "$SERVER_PID_FILE"))"
  else
    echo "web_server: stopped"
  fi

  if pid_running "$WEBAPP_PID_FILE"; then
    echo "webapp: running (pid $(cat "$WEBAPP_PID_FILE"))"
  else
    echo "webapp: stopped"
  fi

  echo
  compose ps
}

cmd_stop() {
  if ! load_env_file; then
    echo "No dev stack has been started for this worktree yet."
    exit 0
  fi

  stop_pid "$WEBAPP_PID_FILE" "webapp"
  stop_pid "$SERVER_PID_FILE" "web_server"

  echo "Stopping Postgres (data volume is preserved)..."
  compose down
}

if [[ $# -ne 1 ]]; then
  usage
fi

case "$1" in
  start) cmd_start ;;
  status) cmd_status ;;
  stop) cmd_stop ;;
  *) usage ;;
esac
