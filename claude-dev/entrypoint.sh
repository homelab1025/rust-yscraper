#!/usr/bin/env bash
set -euo pipefail

# ── Runs as root for exactly this one-shot bootstrap, then execs into
#    the "dev" user for the rest of the container's life (see the final
#    `exec gosu`). No root process lingers after this script returns. ──

PG_CONF="$(find /etc/postgresql -maxdepth 3 -name postgresql.conf | head -n1)"
if [[ -z "$PG_CONF" ]]; then
  echo "❌ Could not locate postgresql.conf" >&2
  exit 1
fi

# ── Ensure PostgreSQL listens on all interfaces ─────────────────────
if ! grep -q "^listen_addresses = '\*'" "$PG_CONF" 2>/dev/null; then
  sed -i "s/^#\?listen_addresses = .*/listen_addresses = '*'/" "$PG_CONF"
  echo "✅ Updated PostgreSQL to listen on all interfaces"
fi

# ── Start PostgreSQL ────────────────────────────────────────────────
echo "🐘 Starting PostgreSQL..."
service postgresql start
for i in $(seq 1 30); do
  pg_isready -q && break
  sleep 1
done
echo "✅ PostgreSQL started on port 5432"

# ── Ensure database exists, and the default "postgres" role has the
#    password config.toml expects (peer auth needs none locally, but the
#    app connects over TCP to "localhost") ──────────────────────────
su - postgres -c "psql -c \"ALTER USER postgres WITH PASSWORD 'postgres';\""
su - postgres -c "psql -tc \"SELECT 1 FROM pg_database WHERE datname='yscraper'\"" | grep -q 1 || \
  su - postgres -c "psql -c \"CREATE DATABASE yscraper OWNER postgres;\""

# ── Apply schema migrations, same changelog docker-compose's liquibase
#    service runs, so the DB ends up in the same state either way ───
liquibase \
  --defaultsFile=/dev/null \
  --classpath=/workspace \
  --changeLogFile=db/changelog/db.changelog-master.yaml \
  --url=jdbc:postgresql://localhost:5432/yscraper \
  --username=postgres \
  --password=postgres \
  --driver=org.postgresql.Driver \
  update
echo "✅ Liquibase migrations applied"

# ── Git identity for commits made inside the container ─────────────
# The SSH key itself needs no setup here - run.sh bind-mounts it
# directly at /home/dev/.ssh/id_ed25519 with the right perms already.
if [[ -n "${GIT_USER_NAME:-}" ]]; then
  gosu dev git config --global user.name "$GIT_USER_NAME"
fi
if [[ -n "${GIT_USER_EMAIL:-}" ]]; then
  gosu dev git config --global user.email "$GIT_USER_EMAIL"
fi

echo "🔐 Dropping to non-root user 'dev'..."
exec gosu dev "$@"
