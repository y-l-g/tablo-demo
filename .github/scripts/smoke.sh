#!/usr/bin/env bash
# Starts the demo built by `topcoat asset bundle` and asks each panel for a page: the panels refuse
# an unsafe declaration when they mount, so a demo that answers has mounted every one.
#
# usage: smoke.sh    (from the repository root, after `topcoat asset bundle`)
set -euo pipefail

base=http://127.0.0.1:3000
log=$(mktemp)
target/debug/tablo-demo >"$log" 2>&1 &
pid=$!
trap 'kill "$pid" 2>/dev/null || true' EXIT

fail() {
    echo "$1" >&2
    cat "$log" >&2
    exit 1
}

for _ in $(seq 120); do
    curl -fs -o /dev/null "$base/admin/login" && break
    kill -0 "$pid" 2>/dev/null || fail "the demo exited before it served"
    sleep 1
done

# Each path, then the status it answers a visitor who is not signed in.
while read -r path status; do
    got=$(curl -s -o /dev/null -w '%{http_code}' "$base$path")
    [ "$got" = "$status" ] || fail "$path answered $got, not $status"
    echo "$path $got"
done <<'PATHS'
/ 200
/admin/login 200
/app/login 200
/admin/audit-log 307
PATHS
