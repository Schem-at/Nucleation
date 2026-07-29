#!/usr/bin/env bash
# Restart the backendless wasm apps on their tailnet ports.
# Each preview MUST launch from its own app dir — launching both from one
# cwd serves the wrong dist (it has happened twice).
set -euo pipefail
cd "$(dirname "$0")"

# The preview serves dist/, and public/engine is copied into dist at BUILD
# time — so syncing a fresh wasm into public/ and restarting silently keeps
# serving the old engine. A whole 28-door batch was once measured against a
# stale wasm this way. Always rebuild.
serve() { # serve <app-dir> <port>
    local app="$1" port="$2"
    if ! (cd "$app" && npm run build >/dev/null 2>&1); then
        echo "$port: !! BUILD FAILED — serving the PREVIOUS dist. Anything you"
        echo "$port: !! measure now is the old build. Fix the build first."
    fi
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
