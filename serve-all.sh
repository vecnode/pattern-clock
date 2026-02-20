#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

cleanup() {
    echo ""
    echo "Stopping servers"
    pkill -f "dx serve" 2>/dev/null || true
    sleep 1
    exit 0
}

trap cleanup SIGINT SIGTERM

echo "Starting pattern-clock"
echo "Web: http://127.0.0.1:8080"
echo "Press Ctrl+C to stop"
echo ""

if command -v gnome-terminal >/dev/null 2>&1; then
    gnome-terminal --title "Web Server" -- bash -c "cd '$SCRIPT_DIR' && dx serve; exec bash" 2>/dev/null &
    sleep 3
    gnome-terminal --title "Desktop Server" -- bash -c "cd '$SCRIPT_DIR' && dx serve --platform desktop; exec bash" 2>/dev/null &
elif command -v xterm >/dev/null 2>&1; then
    xterm -T "Web Server" -e "cd '$SCRIPT_DIR' && dx serve; exec bash" 2>/dev/null &
    sleep 3
    xterm -T "Desktop Server" -e "cd '$SCRIPT_DIR' && dx serve --platform desktop; exec bash" 2>/dev/null &
else
    dx serve &
    sleep 3
    dx serve --platform desktop &
fi

while true; do
    sleep 1
    if ! pgrep -f "dx serve" >/dev/null 2>&1; then
        break
    fi
done
