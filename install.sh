#!/bin/sh
# Install script for Markplane
# Usage: curl -fsSL https://raw.githubusercontent.com/zerowand01/markplane/master/install.sh | sh
#
# Environment variables:
#   INSTALL_DIR  - Installation directory (default: ~/.local/bin)
#   VERSION      - Specific version to install (default: latest)

set -eu

REPO="zerowand01/markplane"
BASE_URL="https://github.com/${REPO}/releases"

main() {
    detect_platform
    resolve_version
    download_and_install
    print_success
}

detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Darwin) ;;
        Linux) ;;
        *)
            echo "Error: Unsupported operating system: $OS" >&2
            echo "Markplane supports macOS and Linux. For Windows, download the zip from:" >&2
            echo "  ${BASE_URL}/latest" >&2
            exit 1
            ;;
    esac

    case "$ARCH" in
        arm64|aarch64)
            if [ "$OS" = "Linux" ]; then
                echo "Error: Unsupported architecture: Linux arm64" >&2
                echo "Pre-built binaries are available for Linux x86_64. Build from source for other architectures:" >&2
                echo "  https://github.com/${REPO}#build-from-source" >&2
                exit 1
            fi
            TARGET="aarch64-apple-darwin"
            ;;
        x86_64)
            case "$OS" in
                Darwin) TARGET="x86_64-apple-darwin" ;;
                Linux)  TARGET="x86_64-unknown-linux-musl" ;;
            esac
            ;;
        *)
            echo "Error: Unsupported architecture: $ARCH" >&2
            echo "Pre-built binaries are available for x86_64 and Apple Silicon. Build from source for other architectures:" >&2
            echo "  https://github.com/${REPO}#build-from-source" >&2
            exit 1
            ;;
    esac

    echo "Detected platform: ${OS} ${ARCH} (${TARGET})"
}

resolve_version() {
    if [ -n "${VERSION:-}" ]; then
        return
    fi

    echo "Fetching latest version..."
    # Follow the redirect from /releases/latest to get the version tag
    VERSION="$(basename "$(curl -fsSL -o /dev/null -w '%{url_effective}' "${BASE_URL}/latest")")"
    if [ -z "$VERSION" ] || [ "$VERSION" = "latest" ]; then
        echo "Error: Could not determine latest version." >&2
        echo "Set VERSION=v0.1.0 to install a specific version, or download manually from:" >&2
        echo "  ${BASE_URL}" >&2
        exit 1
    fi
    echo "Installing markplane ${VERSION}"
}

download_and_install() {
    INSTALL_DIR="${INSTALL_DIR:-${HOME}/.local/bin}"
    ARCHIVE="markplane-${VERSION}-${TARGET}.tar.gz"
    DOWNLOAD_URL="${BASE_URL}/download/${VERSION}/${ARCHIVE}"
    CHECKSUMS_URL="${BASE_URL}/download/${VERSION}/checksums-sha256.txt"

    WORK_DIR="$(mktemp -d)"
    trap 'rm -rf "$WORK_DIR"' EXIT

    echo "Downloading ${ARCHIVE}..."
    curl -fsSL -o "${WORK_DIR}/${ARCHIVE}" "$DOWNLOAD_URL"

    echo "Verifying checksum..."
    curl -fsSL -o "${WORK_DIR}/checksums-sha256.txt" "$CHECKSUMS_URL"

    EXPECTED="$(grep -F "${ARCHIVE}" "${WORK_DIR}/checksums-sha256.txt" | cut -d' ' -f1)"
    if [ -z "$EXPECTED" ]; then
        echo "Error: Could not find checksum for ${ARCHIVE}" >&2
        exit 1
    fi

    # Compute checksum (shasum on macOS, sha256sum on Linux)
    if command -v sha256sum >/dev/null 2>&1; then
        ACTUAL="$(sha256sum "${WORK_DIR}/${ARCHIVE}" | cut -d' ' -f1)"
    elif command -v shasum >/dev/null 2>&1; then
        ACTUAL="$(shasum -a 256 "${WORK_DIR}/${ARCHIVE}" | cut -d' ' -f1)"
    else
        echo "Warning: No sha256sum or shasum found, skipping checksum verification" >&2
        ACTUAL="$EXPECTED"
    fi

    if [ "$EXPECTED" != "$ACTUAL" ]; then
        echo "Error: Checksum verification failed" >&2
        echo "  Expected: ${EXPECTED}" >&2
        echo "  Actual:   ${ACTUAL}" >&2
        exit 1
    fi
    echo "Checksum verified."

    echo "Extracting to ${INSTALL_DIR}..."
    mkdir -p "$INSTALL_DIR"
    tar -xzf "${WORK_DIR}/${ARCHIVE}" -C "$WORK_DIR"
    mv "${WORK_DIR}/markplane" "${INSTALL_DIR}/markplane"
    chmod +x "${INSTALL_DIR}/markplane"
}

print_success() {
    echo ""
    echo "Markplane ${VERSION} installed to ${INSTALL_DIR}/markplane"

    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            echo ""
            echo "Add ${INSTALL_DIR} to your PATH:"
            echo ""
            echo "  echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.bashrc  # or ~/.zshrc"
            ;;
    esac

    echo ""
    echo "Get started:"
    echo "  markplane init --name \"My Project\""
    echo ""
    echo "Documentation: https://github.com/${REPO}"
}

main
