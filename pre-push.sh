#!/bin/bash
# Nucleation Pre-Push Verification — thin wrapper
# The real implementation lives in tools/prepush.py (Rich TUI)
exec python3 "$(dirname "$0")/tools/prepush.py" "$@"
