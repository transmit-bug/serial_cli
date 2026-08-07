#!/usr/bin/env sh
# Serial CLI installer — Linux & macOS
#
# Detects the platform, downloads the matching release binary from GitHub,
# verifies its SHA-256 checksum, and installs it. Optionally registers the
# daemon to auto-start on boot (`--service`).
#
# Usage:
#   install.sh [VERSION] [--service] [--prefix DIR] [--uninstall]
#
# Examples:
#   ./install.sh                        # latest release, no service
#   ./install.sh v0.6.0 --service       # pinned version + auto-start
#   ./install.sh --prefix ~/bin         # install to a custom directory
#   ./install.sh --uninstall            # remove binary (+ service if set)
#
# NOTE: piping curl output straight into sh is convenient but lets the
# script run with your privileges before you see it. Prefer downloading
# the script, reviewing it, then running it.

set -eu

REPO="transmit-bug/serial_cli"
# Override for self-hosted mirrors / testing:
#   SERIAL_CLI_RELEASE_URL=https://api.github.com/repos/transmit-bug/serial_cli/releases/latest
DEFAULT_RELEASE_URL="https://api.github.com/repos/${REPO}/releases"
VERSION="latest"
SERVICE=0
UNINSTALL=0
PREFIX=""

while [ "$#" -gt 0 ]; do
    case "$1" in
        --service) SERVICE=1 ;;
        --uninstall) UNINSTALL=1 ;;
        --prefix) PREFIX="$2"; shift ;;
        --prefix=*) PREFIX="${1#--prefix=}" ;;
        v[0-9]* | [0-9]*.*) VERSION="$1" ;;
        *) echo "Unknown argument: $1" >&2; exit 1 ;;
    esac
    shift
done

if [ "$UNINSTALL" = "1" ]; then
    if command -v serial-cli >/dev/null 2>&1; then
        serial-cli server service uninstall >/dev/null 2>&1 || true
        BIN_PATH="$(command -v serial-cli)"
        rm -f "$BIN_PATH"
        echo "Removed $BIN_PATH"
    else
        echo "serial-cli is not installed."
    fi
    exit 0
fi

# --- Platform detection -----------------------------------------------------
OS="$(uname -s)"
ARCH="$(uname -m)"
case "$OS" in
    Linux) OS_DIR="linux" ;;
    Darwin) OS_DIR="macos" ;;
    *) echo "Unsupported OS: $OS (use Windows? try install.ps1)" >&2; exit 1 ;;
esac
case "$ARCH" in
    x86_64|amd64) ARCH_DIR="x86_64" ;;
    aarch64|arm64) ARCH_DIR="aarch64" ;;
    *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

ASSET_NAME="serial-cli-${OS_DIR}-${ARCH_DIR}"

# --- Resolve release + asset URL -------------------------------------------
if [ "$VERSION" = "latest" ]; then
    API_URL="${SERIAL_CLI_RELEASE_URL:-${DEFAULT_RELEASE_URL}/latest}"
else
    API_URL="${SERIAL_CLI_RELEASE_URL:-${DEFAULT_RELEASE_URL}/tags/${VERSION}}"
fi
echo "Fetching release info from ${API_URL}"
ASSET_URL="$(curl -fsSL "$API_URL" \
    | grep '"browser_download_url"' \
    | sed "s/.*\"browser_download_url\": *\"\([^\"]*\)\".*/\1/" \
    | grep "/${ASSET_NAME}$" \
    | head -1)"
if [ -z "$ASSET_URL" ]; then
    echo "Could not find asset '${ASSET_NAME}' in ${VERSION} release." >&2
    echo "Assets available:" >&2
    curl -fsSL "$API_URL" | grep '"browser_download_url"' | sed "s/.*\"\([^\"]*\)\".*/\1/" >&2 || true
    exit 1
fi
CHECKSUM_URL="${ASSET_URL}.sha256"

# --- Download + verify ------------------------------------------------------
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
echo "Downloading ${ASSET_URL}"
curl -fL "$ASSET_URL" -o "$TMP_DIR/serial-cli"
echo "Downloading checksum"
curl -fsSL "$CHECKSUM_URL" -o "$TMP_DIR/serial-cli.sha256"

EXPECTED="$(awk '{print $1}' "$TMP_DIR/serial-cli.sha256")"
ACTUAL="$( (shasum -a 256 "$TMP_DIR/serial-cli" 2>/dev/null || sha256sum "$TMP_DIR/serial-cli") | awk '{print $1}')"
if [ "$EXPECTED" != "$ACTUAL" ]; then
    echo "Checksum verification FAILED." >&2
    echo "  expected: $EXPECTED" >&2
    echo "  actual:   $ACTUAL" >&2
    exit 1
fi
echo "Checksum verified."

# --- Install ----------------------------------------------------------------
if [ -n "$PREFIX" ]; then
    INSTALL_DIR="$PREFIX"
else
    if [ "$(id -u)" = "0" ]; then
        INSTALL_DIR="/usr/local/bin"
    else
        INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$INSTALL_DIR"
    fi
fi
mkdir -p "$INSTALL_DIR"
chmod +x "$TMP_DIR/serial-cli"
mv "$TMP_DIR/serial-cli" "$INSTALL_DIR/serial-cli"
echo "Installed to $INSTALL_DIR/serial-cli"

if ! echo ":$PATH:" | grep -q ":$INSTALL_DIR:"; then
    echo "NOTE: add $INSTALL_DIR to your PATH, e.g.:"
    echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc"
fi

# --- Optional auto-start ----------------------------------------------------
if [ "$SERVICE" = "1" ]; then
    echo "Registering daemon auto-start..."
    "$INSTALL_DIR/serial-cli" server service install
fi

echo
echo "Done. Run 'serial-cli --help' to get started."
