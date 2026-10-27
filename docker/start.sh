#!/usr/bin/env bash
set -Eeuo pipefail

# Allow one-off commands to bypass normal application startup.
if [[ $# -gt 0 ]]; then
    exec "$@"
fi

# Abort startup if pending migrations cannot be applied.
rincuma-migrate migration apply

api_pid=""
web_pid=""

stop_services() {
    # Disable traps before stopping children to avoid recursive cleanup.
    trap - EXIT INT TERM
    for pid in "$api_pid" "$web_pid"; do
        if [[ -n "$pid" ]]; then
            kill -TERM "$pid" 2>/dev/null || true
        fi
    done
    wait || true
}

# Always stop both services, including when Docker sends a shutdown signal.
trap stop_services EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

rincuma-api &
api_pid=$!
node /app/web/server/index.mjs &
web_pid=$!

# Stop both services if either process exits, so Compose can restart the app.
set +e
wait -n "$api_pid" "$web_pid"
status=$?
set -e
exit "$status"
