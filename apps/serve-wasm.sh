#!/usr/bin/env bash
# Restart the backendless wasm apps on their tailnet ports.
# Each preview MUST launch from its own app dir — launching both from one
# cwd serves the wrong dist (it has happened twice).
set -euo pipefail
cd "$(dirname "$0")"

serve() { # serve <app-dir> <port>
    local app="$1" port="$2"
    kill "$(lsof -nP -tiTCP:"$port" -sTCP:LISTEN 2>/dev/null)" 2>/dev/null || true
    sleep 1
    (cd "$app" && nohup npx vite preview --host 0.0.0.0 --port "$port" --strictPort \
        > preview.log 2>&1 & echo $! > preview.pid)
}

serve door-cert-wasm 8433
serve flying-ga-wasm 8444
sleep 3
for port in 8433 8444; do
    title=$(curl -s "http://127.0.0.1:$port/" | grep -o '<title>[^<]*' | cut -c8-)
    echo "$port: ${title:-NOT RESPONDING}"
done
