#!/bin/bash
set -euo pipefail

WORKSPACE="/Users/johnny/.ai/smidja/yggdrasil"
BUNDLE_DIR="$WORKSPACE/target/release/bundle/macos"
DEST="/Applications"

APPS=(hlidskjalf svalinn kvasir ratatoskr yggdrasil)

# Build all apps
for app in "${APPS[@]}"; do
    echo "==> Building $app"
    cd "$WORKSPACE/$app"
    npx tauri build 2>&1 | tail -3
done

# Move to /Applications (remove old first)
for app in "${APPS[@]}"; do
    src="$BUNDLE_DIR/$app.app"
    dst="$DEST/$app.app"

    if [ ! -d "$src" ]; then
        echo "!! $src not found, skipping"
        continue
    fi

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
