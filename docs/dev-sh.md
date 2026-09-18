# dev.sh

`dev.sh` manages the full local dev stack (Postgres + Liquibase migrations + `web_server` + webapp) for the
current git worktree. Ports and the compose project name are derived from the worktree's absolute path, so
multiple worktrees can each run their own stack at the same time without colliding.

This document must be kept in sync with `dev.sh` — see `CONTRIBUTING.md`.

## Flow

```
dev.sh {start|status|stop}
│
├─ compute_ports()            # cksum(worktree path) % 900 → OFFSET
│    SERVER_PORT = 3000+OFF   DB_PORT = 5432+OFF
│    ADMINER_PORT = 8080+OFF  WEB_PORT = 5173+OFF
│    COMPOSE_PROJECT_NAME = yscraper-$OFFSET
│
├─ write_env_file()           # .dev/env (persisted for status/stop)
│
└─ dispatch on $1
   │
   ├─ start ──────────────────────────────────────────────
   │   │
   │   ├─ cp conf/config.toml → config.toml   (if missing)
   │   │
   │   ├─ compose up -d --wait postgres        (docker)
   │   ├─ compose up --no-deps liquibase       (migrations)
   │   │
   │   ├─ web_server? ── pid_running(.dev/server.pid)?
   │   │     ├─ yes → skip
   │   │     └─ no  → nohup cargo run -p web_server
   │   │               YSCR_PORT / YSCR_DB_PORT
   │   │               log → .dev/server.log, pid → .dev/server.pid
   │   │
   │   └─ webapp? ── pid_running(.dev/webapp.pid)?
   │         ├─ yes → skip
   │         └─ no  → npm install (if no node_modules)
   │                   nohup npm run dev
   │                   API_PORT / WEB_PORT
   │                   log → .dev/webapp.log, pid → .dev/webapp.pid
   │
   ├─ status ─────────────────────────────────────────────
   │   │
   │   ├─ load_env_file() ── no .dev/env → "run start", exit 1
   │   ├─ print_ports
   │   ├─ pid_running(server.pid)  → running/stopped
   │   ├─ pid_running(webapp.pid)  → running/stopped
   │   └─ compose ps
   │
   └─ stop ───────────────────────────────────────────────
       │
       ├─ load_env_file() ── no .dev/env → exit 0
       ├─ stop_pid(webapp.pid)   # pkill children + TERM, rm pid file
       ├─ stop_pid(server.pid)
       └─ compose down           # Postgres data volume preserved
```

## State

All state lives in `.dev/` (gitignored):

| File | Purpose |
|------|---------|
| `env` | Persisted ports and compose project name, loaded by `status`/`stop` |
| `server.pid` | PID of the backgrounded `cargo run -p web_server` |
| `webapp.pid` | PID of the backgrounded `npm run dev` |
| `server.log` | `web_server` stdout/stderr |
| `webapp.log` | webapp stdout/stderr |
