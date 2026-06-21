#!/usr/bin/env bash
# install.sh — install musk (auto-musk CLI agent) from source.
#
# musk depends on auto-ai-agent/auto-ai-client (from auto-ai repo) which
# depend on ai-config → auto-atom/auto-val (from auto-lang repo), all via
# path deps. This script clones the sibling repos to the expected relative
# locations and runs `cargo install --path`.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/auto-stack/auto-musk/rust-impl/install.sh | bash
#   # or:
#   git clone https://github.com/auto-stack/auto-musk.git && cd auto-musk && bash install.sh
#
# Prerequisites: Rust (rustup), git, a running auto-ai-daemon (aaid).

set -euo pipefail

# Colors
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
info()  { echo -e "${GREEN}✓${NC} $*"; }
warn()  { echo -e "${YELLOW}!${NC} $*"; }
error() { echo -e "${RED}✗${NC} $*"; exit 1; }

# Detect parent dir (where sibling repos go).
PARENT="$(cd "$(dirname "$0")/.." && pwd)"
MUSK_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "Installing musk from $MUSK_DIR"
echo "Sibling repos will be in $PARENT"
echo ""

# --- 1. Clone / update sibling repos ------------------------------------------

clone_or_update() {
    local repo="$1" dir="$2" branch="${3:-main}"
    if [ -d "$dir" ]; then
        info "$repo already exists at $dir, pulling latest..."
        git -C "$dir" fetch origin "$branch"
        git -C "$dir" checkout "$branch"
        git -C "$dir" reset --hard "origin/$branch"
    else
        info "Cloning $repo → $dir"
        git clone --depth 1 -b "$branch" "$repo" "$dir"
    fi
}

clone_or_update "https://github.com/auto-stack/auto-ai.git" \
    "$PARENT/auto-ai" main
clone_or_update "https://gitee.com/auto-stack/auto-lang.git" \
    "$PARENT/auto-lang" master

# --- 2. Build + install musk --------------------------------------------------

info "Building and installing musk..."
cd "$MUSK_DIR/backend"
cargo install --path crates/musk --force

# --- 3. Verify ----------------------------------------------------------------

info "Verifying installation..."
if command -v musk &>/dev/null; then
    musk --version
    echo ""
    info "musk installed successfully!"
    echo ""
    echo "Next steps:"
    echo "  1. Build & start the LLM daemon:"
    echo "     cd $PARENT/auto-ai && cargo run -p auto-ai-daemon"
    echo "  2. Configure ~/.config/autoos/ai-daemon.at (provider + API key)"
    echo "  3. Run: musk run \"your task\""
    echo "     Or interactive: musk chat"
else
    warn "musk binary not found in PATH. Check ~/.cargo/bin/ is on your PATH."
fi
