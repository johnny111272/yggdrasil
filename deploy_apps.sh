#!/bin/bash
set -euo pipefail

WORKSPACE="/Users/johnny/.ai/smidja/yggdrasil"
BUNDLE_DIR="$WORKSPACE/target/release/bundle/macos"
DEST="/Applications"

ALL_APPS=(hlidskjalf svalinn kvasir ratatoskr yggdrasil)

if [ $# -gt 0 ]; then
    APPS=("$@")
else
    APPS=("${ALL_APPS[@]}")
fi

# Build all apps
for app in "${APPS[@]}"; do
    echo "==> Building $app"
    cd "$WORKSPACE/$app"
    npx tauri build 2>&1 | tail -3
done

# Kill running apps, then move to /Applications
for app in "${APPS[@]}"; do
    src="$BUNDLE_DIR/$app.app"
    dst="$DEST/$app.app"

    if [ ! -d "$src" ]; then
        echo "!! $src not found, skipping"
        continue
    fi

    # Kill running process before overwriting
    if pgrep -x "$app" >/dev/null 2>&1; then
        echo "==> Killing running $app"
        pkill -x "$app" || true
        sleep 0.5
    fi

    # Clear WebKit caches (WKWebView serves stale frontend otherwise)
    identifier="com.johnny.$app"
    rm -rf ~/Library/WebKit/"$app" ~/Library/WebKit/"$identifier"
    rm -rf ~/Library/Caches/"$app" ~/Library/Caches/"$identifier"

    # Remove old (symlink or directory)
    if [ -e "$dst" ] || [ -L "$dst" ]; then
        echo "==> Removing old $dst"
        rm -rf "$dst"
    fi

    echo "==> Moving $app.app to $DEST"
    mv "$src" "$dst"
done

# Clean build artifacts to reclaim disk space
echo ""
echo "==> Cleaning build artifacts"
rm -rf "$WORKSPACE/target/release/bundle"
rm -rf "$WORKSPACE/target/release/deps"
rm -rf "$WORKSPACE/target/release/build"
rm -rf "$WORKSPACE/target/release/.fingerprint"

# Show result
echo ""
echo "Done. Deployed:"
ls -ld /Applications/{hlidskjalf,svalinn,kvasir,ratatoskr,yggdrasil}.app 2>/dev/null

echo ""
du -sh "$WORKSPACE/target" 2>/dev/null || true
