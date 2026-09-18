#!/usr/bin/env bash
set -euo pipefail

# Builds and runs the claude-dev container with only the workspace (this
# repo) and the network exposed to it: no other host mounts, no docker
# socket, capabilities dropped to the minimum Postgres' startup needs.

cd "$(dirname "$0")/.."

IMAGE=claude-dev
# Shared across worktrees on purpose: this is the container's /home/dev/.claude,
# and sharing it keeps the OAuth login and memory in sync across concurrent
# sessions - the same way multiple `claude` terminals on one host normally
# share ~/.claude. Only the ports below need to be per-worktree.
VOLUME=claude-dev-home

# Derive a stable per-worktree port offset from this worktree's absolute path
# (same formula as dev.sh), so several worktrees can each run their own
# claude-dev container at the same time without their published host ports
# colliding. The container's internal ports (3000/5173/5432) stay fixed -
# only the host side of the `-p` mapping below changes.
REPO_DIR="$(pwd)"
CHECKSUM="$(printf '%s' "$REPO_DIR" | cksum | cut -d' ' -f1)"
OFFSET=$((10#$CHECKSUM % 900))
HOST_SERVER_PORT=$((3000 + OFFSET))
HOST_WEB_PORT=$((5173 + OFFSET))
HOST_DB_PORT=$((5432 + OFFSET))
CONTAINER_NAME="claude-dev-$OFFSET"

docker build \
  --build-arg UID="$(id -u)" \
  --build-arg GID="$(id -g)" \
  -t "$IMAGE" \
  claude-dev/

# Named volume (Docker-managed, not a host path) so the Claude Code OAuth
# login persists across container restarts.
docker volume create "$VOLUME" > /dev/null

# Optional: mount a dedicated SSH deploy key from the host so the container
# can push to GitHub, without ever baking the key into the image. Bind
# mounts keep the host file's owner/perms as-is, and the image is built
# with matching UID/GID, so this only works cleanly if the key is chmod 600
# on the host already.
SSH_MOUNT_ARGS=()
if [[ -n "${CLAUDE_DEV_SSH_KEY:-}" ]]; then
  if [[ ! -f "$CLAUDE_DEV_SSH_KEY" ]]; then
    echo "❌ CLAUDE_DEV_SSH_KEY is set to '$CLAUDE_DEV_SSH_KEY' but that file doesn't exist" >&2
    exit 1
  fi
  KEY_PERMS=$(stat -c '%a' "$CLAUDE_DEV_SSH_KEY" 2>/dev/null || stat -f '%Lp' "$CLAUDE_DEV_SSH_KEY")
  if [[ "$KEY_PERMS" != "600" ]]; then
    echo "⚠️  $CLAUDE_DEV_SSH_KEY has permissions $KEY_PERMS, not 600 - ssh inside the container will reject it as-is." >&2
    echo "    Fix with: chmod 600 $CLAUDE_DEV_SSH_KEY" >&2
  fi
  SSH_MOUNT_ARGS=(-v "$CLAUDE_DEV_SSH_KEY":/home/dev/.ssh/id_ed25519:ro)
fi

echo "Worktree: $REPO_DIR"
echo "Host ports — backend: $HOST_SERVER_PORT  frontend: $HOST_WEB_PORT  db: $HOST_DB_PORT  (container: $CONTAINER_NAME)"
echo

docker run -it --rm \
  --name "$CONTAINER_NAME" \
  -v "$(pwd)":/workspace \
  -v "$VOLUME":/home/dev/.claude \
  "${SSH_MOUNT_ARGS[@]}" \
  -e GIT_USER_NAME="${GIT_USER_NAME:-claude}" \
  -e GIT_USER_EMAIL="${GIT_USER_EMAIL:-florin.diaconeasa@gmail.com}" \
  -e HOST_SERVER_PORT="$HOST_SERVER_PORT" \
  -e HOST_WEB_PORT="$HOST_WEB_PORT" \
  -e HOST_DB_PORT="$HOST_DB_PORT" \
  -p "$HOST_WEB_PORT":5173 \
  -p "$HOST_SERVER_PORT":3000 \
  -p "$HOST_DB_PORT":5432 \
  --cap-drop=ALL \
  --cap-add=SETUID \
  --cap-add=SETGID \
  --cap-add=CHOWN \
  --cap-add=DAC_OVERRIDE \
  --cap-add=FOWNER \
  --security-opt=no-new-privileges=true \
  "$IMAGE" "$@"
