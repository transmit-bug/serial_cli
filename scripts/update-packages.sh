#!/usr/bin/env bash
set -euo pipefail

# Serial CLI Package Manager Update Script
# Updates AUR, Homebrew, and Scoop repositories after a release
# Usage: ./scripts/update-packages.sh <version> <sha256> [--aur-only] [--brew-only] [--scoop-only]

VERSION="${1:-}"
SHA256="${2:-}"

AUR_ONLY=false
BREW_ONLY=false
SCOOP_ONLY=false

for arg in "${@:3}"; do
    case "$arg" in
        --aur-only) AUR_ONLY=true ;;
        --brew-only) BREW_ONLY=true ;;
        --scoop-only) SCOOP_ONLY=true ;;
    esac
done

if [ -z "$VERSION" ] || [ -z "$SHA256" ]; then
    echo "Usage: update-packages.sh <version> <sha256>"
    echo "Example: update-packages.sh v1.2.3 abc123..."
    exit 1
fi

VERSION_NUM="${VERSION#v}"

generate_aur() {
    local output_dir="aur"
    mkdir -p "$output_dir"
    echo "Generating AUR files..."

    cat > "$output_dir/PKGBUILD" << EOF
pkgname=serial-cli
pkgver=$VERSION_NUM
pkgrel=1
pkgdesc="Universal serial port CLI tool optimized for AI interaction"
arch=('x86_64' 'aarch64')
url="https://github.com/transmit-bug/serial_cli"
license=('MIT' 'Apache')
depends=('gcc-libs')
makedepends=('cargo')
source=("\$pkgname-\$pkgver.tar.gz::https://github.com/transmit-bug/serial_cli/archive/refs/tags/v\$pkgver.tar.gz")
sha256sums=('$SHA256')

build() {
  cd "\$pkgname-\$pkgver"
  cargo build --release
}

package() {
  cd "\$pkgname-\$pkgver"
  install -Dm755 "target/release/serial-cli" "\$pkgdir/usr/bin/serial-cli"
}
EOF

    cat > "$output_dir/.SRCINFO" << EOF
pkgbase = serial-cli
pkgdesc = Universal serial port CLI tool optimized for AI interaction
pkgver = $VERSION_NUM
pkgrel = 1
url = https://github.com/transmit-bug/serial_cli
arch = x86_64
arch = aarch64
license = MIT
license = Apache
depends = gcc-libs
makedepends = cargo
source = serial-cli-$VERSION_NUM.tar.gz::https://github.com/transmit-bug/serial_cli/archive/refs/tags/v$VERSION_NUM.tar.gz
sha256sums = $SHA256
EOF

    echo "  Generated $output_dir/PKGBUILD and .SRCINFO"
    echo "  To submit: cd $output_dir && git clone ssh://aur@aur.archlinux.org/serial-cli.git"
}

update_homebrew() {
    local tap_repo="https://github.com/zazac-zhang/serial-cli-homebrew.git"
    local tmp_dir
    tmp_dir=$(mktemp -d)

    echo "Cloning Homebrew tap repository..."
    git clone "$tap_repo" "$tmp_dir"
    cd "$tmp_dir"

    local formula_file="Formula/serial-cli.rb"
    if [ -f "$formula_file" ]; then
        sed -i.bak "s/v{version}/$VERSION/g" "$formula_file"
        sed -i.bak "s/{sha256_from_release}/$SHA256/g" "$formula_file"
        rm -f "$formula_file.bak"
        echo "  Updated existing formula"
    else
        mkdir -p Formula
        cat > "$formula_file" << EOF
class SerialCli < Formula
  desc "Universal serial port CLI tool optimized for AI interaction"
  homepage "https://github.com/transmit-bug/serial_cli"
  url "https://github.com/transmit-bug/serial_cli/archive/refs/tags/${VERSION}.tar.gz"
  sha256 "$SHA256"
  license any_of: ["MIT", "Apache-2.0"]

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--path", "."
  end

  test do
    system "#{bin}/serial-cli", "--version"
  end
end
EOF
        echo "  Created new formula"
    fi

    git config user.name "github-actions[bot]"
    git config user.email "github-actions[bot]@users.noreply.github.com"
    git add "$formula_file"
    git commit -m "Bump to $VERSION"
    git push
    echo "  Homebrew tap updated"

    rm -rf "$tmp_dir"
}

update_scoop() {
    local bucket_repo="https://github.com/zazac-zhang/serial-cli-scoop.git"
    local tmp_dir
    tmp_dir=$(mktemp -d)

    echo "Cloning Scoop bucket repository..."
    git clone "$bucket_repo" "$tmp_dir"
    cd "$tmp_dir"

    local manifest_file="bucket/serial-cli.json"
    mkdir -p bucket
    cat > "$manifest_file" << EOF
{
  "version": "$VERSION_NUM",
  "description": "Universal serial port CLI tool",
  "homepage": "https://github.com/transmit-bug/serial_cli",
  "license": "MIT OR Apache-2.0",
  "url": "https://github.com/transmit-bug/serial_cli/releases/download/${VERSION}/serial-cli-windows-x86_64.exe",
  "hash": "$SHA256",
  "bin": "serial-cli.exe",
  "post_install": [
    "Write-Host \"Serial CLI $VERSION_NUM installed successfully!\""
  ]
}
EOF

    git config user.name "github-actions[bot]"
    git config user.email "github-actions[bot]@users.noreply.github.com"
    git add "$manifest_file"
    git commit -m "Bump to $VERSION_NUM"
    git push
    echo "  Scoop bucket updated"

    rm -rf "$tmp_dir"
}

echo "Updating package managers for $VERSION..."
echo ""

if [ "$BREW_ONLY" = false ] && [ "$SCOOP_ONLY" = false ]; then
    generate_aur
fi

if [ "$AUR_ONLY" = false ] && [ "$SCOOP_ONLY" = false ]; then
    echo ""
    update_homebrew
fi

if [ "$AUR_ONLY" = false ] && [ "$BREW_ONLY" = false ]; then
    echo ""
    update_scoop
fi

echo ""
echo "✅ All package managers updated."
