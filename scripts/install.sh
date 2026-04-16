#!/bin/sh
# beck installer — downloads the latest release binary for your platform.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/notabotchef/beck/main/scripts/install.sh | sh
#
# Or download and inspect first:
#   curl -fsSL ...install.sh -o install.sh
#   sh install.sh

set -eu

REPO="notabotchef/beck"
INSTALL_DIR="${BECK_INSTALL_DIR:-/usr/local/bin}"
BINARY="beck"

# ── detect platform ─────────────────────────────────────────────────
detect_target() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)  OS_TAG="linux" ;;
        Darwin) OS_TAG="macos" ;;
        *)
            echo "error: unsupported OS: $OS" >&2
            exit 1
            ;;
    esac

    case "$ARCH" in
        x86_64|amd64)  ARCH_TAG="x86_64" ;;
        aarch64|arm64) ARCH_TAG="aarch64" ;;
        *)
            echo "error: unsupported architecture: $ARCH" >&2
            exit 1
            ;;
    esac

    # prefer musl on linux
    if [ "$OS_TAG" = "linux" ]; then
        if ldd --version 2>&1 | grep -qi musl; then
            TARGET="${ARCH_TAG}-unknown-linux-musl"
        else
            TARGET="${ARCH_TAG}-unknown-linux-gnu"
        fi
    else
        TARGET="${ARCH_TAG}-apple-darwin"
    fi

    # map to release archive name
    case "$TARGET" in
        x86_64-unknown-linux-gnu)  ARCHIVE_NAME="beck-*-linux-x86_64-gnu" ;;
        x86_64-unknown-linux-musl) ARCHIVE_NAME="beck-*-linux-x86_64-musl" ;;
        aarch64-unknown-linux-gnu) ARCHIVE_NAME="beck-*-linux-aarch64" ;;
        x86_64-apple-darwin)       ARCHIVE_NAME="beck-*-macos-x86_64" ;;
        aarch64-apple-darwin)      ARCHIVE_NAME="beck-*-macos-aarch64" ;;
        *)
            echo "error: no build for target $TARGET" >&2
            exit 1
            ;;
    esac
}

# ── get latest release tag ──────────────────────────────────────────
get_latest_tag() {
    TAG=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' | head -1 | cut -d'"' -f4)
    if [ -z "$TAG" ]; then
        echo "error: could not fetch latest release" >&2
        exit 1
    fi
    echo "$TAG"
}

# ── download and verify ─────────────────────────────────────────────
download_and_install() {
    TAG="$1"
    VERSION="${TAG#v}"
    PATTERN=$(echo "$ARCHIVE_NAME" | sed "s/\*/${TAG}/g")

    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${TAG}/${PATTERN}.tar.gz"
    SHA_URL="${DOWNLOAD_URL}.sha256"

    TMPDIR=$(mktemp -d)
    trap 'rm -rf "$TMPDIR"' EXIT

    echo "Downloading beck ${TAG} for ${TARGET}..."
    curl -fsSL "$DOWNLOAD_URL" -o "${TMPDIR}/beck.tar.gz"
    curl -fsSL "$SHA_URL" -o "${TMPDIR}/beck.tar.gz.sha256"

    echo "Verifying checksum..."
    cd "$TMPDIR"
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum -c beck.tar.gz.sha256
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 -c beck.tar.gz.sha256
    else
        echo "warning: no sha256 verification tool found, skipping check"
    fi

    echo "Extracting..."
    tar xzf beck.tar.gz

    echo "Installing to ${INSTALL_DIR}/beck..."
    if [ -w "$INSTALL_DIR" ]; then
        mv beck "$INSTALL_DIR/beck"
        chmod +x "$INSTALL_DIR/beck"
    else
        echo "(needs sudo for ${INSTALL_DIR})"
        sudo mv beck "$INSTALL_DIR/beck"
        sudo chmod +x "$INSTALL_DIR/beck"
    fi
}

# ── main ─────────────────────────────────────────────────────────────
main() {
    detect_target
    TAG=$(get_latest_tag)
    download_and_install "$TAG"

    echo ""
    echo "beck ${TAG} installed successfully."
    echo ""
    "$INSTALL_DIR/$BINARY" --version
    echo ""
    echo "Run 'beck sync' to index your skills, then 'beck prompt' for agent integration."
}

main
