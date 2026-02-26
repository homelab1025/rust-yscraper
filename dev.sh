#!/usr/bin/env bash
set -euo pipefail

SESSION="dev"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
  echo "Usage: $(basename "$0") {start|status|stop}"
  echo
  echo "Commands:"
  echo "  start   Create a tmux session with docker-compose, web_server, and webapp"
  echo "  status  Show the status of the dev tmux session and its panes"
  echo "  stop    Kill the dev tmux session and all processes within it"
  exit 1
}

cmd_start() {
  if tmux has-session -t "$SESSION" 2>/dev/null; then
    echo "Session '$SESSION' already exists. Use 'status' to inspect or 'stop' to tear down."
    exit 1
  fi

  echo "Starting dev stack in tmux session '$SESSION'..."

  # Pane 0: docker-compose
  tmux new-session -d -s "$SESSION" -c "$SCRIPT_DIR"
  tmux rename-window -t "$SESSION:0" 'dev-stack'
  tmux send-keys -t "$SESSION:0.0" "docker-compose up" C-m

  # Pane 1: web_server (wait for infra to be ready)
  tmux split-window -h -t "$SESSION:0" -c "$SCRIPT_DIR"
  tmux send-keys -t "$SESSION:0.1" "sleep 5 && cargo run -p web_server" C-m

  # Pane 2: webapp dev server
  tmux split-window -v -t "$SESSION:0.1" -c "$SCRIPT_DIR"
  tmux send-keys -t "$SESSION:0.2" "cd webapp && npm run dev" C-m

  echo "Dev stack started. Attach with: tmux attach -t $SESSION"
}

cmd_status() {
  if ! tmux has-session -t "$SESSION" 2>/dev/null; then
    echo "Session '$SESSION' is not running."
    exit 1
  fi

  echo "Session '$SESSION' is running."
  echo
  echo "Panes:"
  tmux list-panes -t "$SESSION" -F \
    "  #{pane_index}: #{pane_current_command}  (pid #{pane_pid}, #{pane_width}x#{pane_height})"
  echo
  echo "Attach with: tmux attach -t $SESSION"
}

cmd_stop() {
  if ! tmux has-session -t "$SESSION" 2>/dev/null; then
    echo "Session '$SESSION' is not running."
    exit 0
  fi

  echo "Stopping processes in dev stack..."
  for pane in $(tmux list-panes -t "$SESSION" -F "#{pane_id}"); do
    tmux send-keys -t "$pane" C-c
  done

  echo "Waiting for processes to exit gracefully..."
  sleep 5

  echo "Killing tmux session..."
  tmux kill-session -t "$SESSION"
  echo "Session '$SESSION' destroyed."
}

# --- main ---
if [[ $# -ne 1 ]]; then
  usage
fi

case "$1" in
  start)  cmd_start  ;;
  status) cmd_status ;;
  stop)   cmd_stop   ;;
  *)      usage      ;;
esac
