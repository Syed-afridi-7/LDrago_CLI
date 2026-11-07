#!/usr/bin/env bash
# LDrago CLI Installer for macOS and Linux
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/Syed-afridi-7/LDrago_CLI/main/install.sh | bash
#   or: ./install.sh

set -e

REPO="Syed-afridi-7/LDrago_CLI"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.ldrago/bin}"
VERSION="${VERSION:-latest}"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${CYAN}==> Initializing LDrago CLI installation...${NC}"

# 1. Detect Operating System
OS_RAW="$(uname -s)"
case "${OS_RAW}" in
    Linux*)     OS="linux" ;;
    Darwin*)    OS="macos" ;;
    *)
        echo -e "${RED}Error: Unsupported operating system: ${OS_RAW}${NC}" >&2
        echo -e "LDrago currently supports Linux and macOS (Darwin)." >&2
        exit 1
        ;;
esac

# 2. Detect Architecture
ARCH_RAW="$(uname -m)"
case "${ARCH_RAW}" in
    x86_64|amd64)   ARCH="x86_64" ;;
    arm64|aarch64)  ARCH="aarch64" ;;
    *)
        echo -e "${RED}Error: Unsupported architecture: ${ARCH_RAW}${NC}" >&2
        exit 1
        ;;
esac

echo -e "${CYAN}==> Detected Platform: ${OS}-${ARCH}${NC}"

# 3. Create install directory
INSTALL_DIR="${INSTALL_DIR%/}"
mkdir -p "$INSTALL_DIR"
TARGET_BIN="$INSTALL_DIR/ldrago"

ARTIFACT_NAME="ldrago-${OS}-${ARCH}.tar.gz"

if [ "$VERSION" = "latest" ]; then
    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ARTIFACT_NAME}"
else
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARTIFACT_NAME}"
fi

DOWNLOAD_SUCCESS=0

# 4. Attempt binary download if not explicitly building from source
if [ "${BUILD_FROM_SOURCE:-0}" != "1" ]; then
    echo -e "${CYAN}==> Attempting to download ${ARTIFACT_NAME}...${NC}"
    echo -e "    ${DOWNLOAD_URL}"

    TMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t 'ldrago')"
    trap 'rm -rf "$TMP_DIR"' EXIT INT TERM
    TMP_ARCHIVE="${TMP_DIR}/${ARTIFACT_NAME}"
    TMP_SHA="${TMP_ARCHIVE}.sha256"

    # Download without eval
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL --proto '=https' --tlsv1.2 -o "$TMP_ARCHIVE" "$DOWNLOAD_URL" 2>/dev/null || true
        curl -fsSL --proto '=https' --tlsv1.2 -o "$TMP_SHA" "${DOWNLOAD_URL}.sha256" 2>/dev/null || true
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$TMP_ARCHIVE" "$DOWNLOAD_URL" 2>/dev/null || true
        wget -qO "$TMP_SHA" "${DOWNLOAD_URL}.sha256" 2>/dev/null || true
    fi

    # Verify checksum if .sha256 is present
    CHECKSUM_VALID=1
    if [ -f "$TMP_ARCHIVE" ] && [ -f "$TMP_SHA" ]; then
        EXPECTED_HASH="$(awk '{print $1}' "$TMP_SHA")"
        ACTUAL_HASH=""
        if command -v sha256sum >/dev/null 2>&1; then
            ACTUAL_HASH="$(sha256sum "$TMP_ARCHIVE" | awk '{print $1}')"
        elif command -v shasum >/dev/null 2>&1; then
            ACTUAL_HASH="$(shasum -a 256 "$TMP_ARCHIVE" | awk '{print $1}')"
        fi

        if [ -n "$ACTUAL_HASH" ] && [ "$EXPECTED_HASH" != "$ACTUAL_HASH" ]; then
            echo -e "${RED}Error: SHA256 checksum mismatch! Expected: ${EXPECTED_HASH}, Actual: ${ACTUAL_HASH}${NC}" >&2
            CHECKSUM_VALID=0
            rm -f "$TMP_ARCHIVE"
        elif [ -n "$ACTUAL_HASH" ]; then
            echo -e "${GREEN}==> Verified SHA256 checksum: ${ACTUAL_HASH}${NC}"
        fi
    fi

    if [ "$CHECKSUM_VALID" -eq 1 ] && [ -f "$TMP_ARCHIVE" ]; then
        if tar -xzf "$TMP_ARCHIVE" -C "$TMP_DIR" 2>/dev/null; then
            FOUND_BIN="$(find "$TMP_DIR" -name ldrago -type f | head -n 1)"
            if [ -n "$FOUND_BIN" ] && [ -f "$FOUND_BIN" ]; then
                cp "$FOUND_BIN" "$TARGET_BIN"
                chmod +x "$TARGET_BIN"
                DOWNLOAD_SUCCESS=1
                echo -e "${GREEN}==> Downloaded and installed binary to: ${TARGET_BIN}${NC}"
            fi
        fi
    fi
    rm -rf "$TMP_DIR"
    trap - EXIT INT TERM
fi

# 5. Fallback: Build from source if download failed or BUILD_FROM_SOURCE=1
if [ "$DOWNLOAD_SUCCESS" -ne 1 ]; then
    echo -e "${YELLOW}==> Pre-built binary not found or build requested.${NC}"
    ROOT_DIR="$(dirname "$INSTALL_DIR")"

    # Check for local workspace
    if [ -f "crates/ld-cli/Cargo.toml" ]; then
        echo -e "${CYAN}==> Local LDrago workspace detected. Building via Cargo...${NC}"
        if ! command -v cargo >/dev/null 2>&1; then
            echo -e "${RED}Error: Cargo is not installed. Please install Rust from https://rustup.rs${NC}" >&2
            exit 1
        fi

        cargo install --path crates/ld-cli --bin ldrago --root "$ROOT_DIR" --force

        if [ ! -f "$TARGET_BIN" ]; then
            if [ -f "target/release/ldrago" ]; then
                cp "target/release/ldrago" "$TARGET_BIN"
            elif [ -f "target/debug/ldrago" ]; then
                cp "target/debug/ldrago" "$TARGET_BIN"
            fi
        fi
    elif command -v cargo >/dev/null 2>&1; then
        echo -e "${CYAN}==> Building from remote git repository via cargo install...${NC}"
        cargo install --git "https://github.com/${REPO}.git" --bin ldrago --root "$ROOT_DIR" --force
    else
        echo -e "${RED}Error: Pre-built release download failed and Cargo is not installed.${NC}" >&2
        echo -e "Please install Rust/Cargo (https://rustup.rs) or ensure network connectivity." >&2
        exit 1
    fi
fi

# 6. Ensure executable permissions
chmod +x "$TARGET_BIN"

# 7. Validate installation
echo -e "${CYAN}==> Validating installation...${NC}"
if [ ! -f "$TARGET_BIN" ]; then
    echo -e "${RED}Error: Target binary not found at ${TARGET_BIN}${NC}" >&2
    exit 1
fi

VERSION_OUTPUT="$("$TARGET_BIN" --version 2>&1)"
echo -e "${GREEN}==> Validation successful: ${VERSION_OUTPUT}${NC}"

# 8. Check PATH
PATH_INCLUDED=0
case ":$PATH:" in
    *":$INSTALL_DIR:"*) PATH_INCLUDED=1 ;;
    *) PATH_INCLUDED=0 ;;
esac

# 9. Success banner & tips
echo ""
echo -e "${GREEN}==========================================================${NC}"
echo -e "${BOLD}${GREEN}         LDrago CLI Installed Successfully!               ${NC}"
echo -e "${GREEN}==========================================================${NC}"
echo -e " Binary location: ${TARGET_BIN}"
echo -e " Version:         ${VERSION_OUTPUT}"
echo ""

if [ "$PATH_INCLUDED" -eq 1 ]; then
    echo -e " ${GREEN}PATH status:     ${INSTALL_DIR} is already in your PATH.${NC}"
else
    echo -e " ${YELLOW}PATH notice:     ${INSTALL_DIR} is NOT in your PATH.${NC}"
    echo -e " To run 'ldrago' from any terminal, add it to your shell configuration:"
    echo ""
    if [ -n "$ZSH_VERSION" ] || [ -f "$HOME/.zshrc" ]; then
        echo -e "   echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.zshrc"
        echo -e "   source ~/.zshrc"
    elif [ -f "$HOME/.bashrc" ]; then
        echo -e "   echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.bashrc"
        echo -e "   source ~/.bashrc"
    else
        echo -e "   echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.profile"
        echo -e "   source ~/.profile"
    fi
fi

echo ""
echo -e "${CYAN}${BOLD}Getting Started:${NC}"
echo -e "  ${BOLD}ldrago doctor${NC}      Validate system environment, models, and sandboxes"
echo -e "  ${BOLD}ldrago tui${NC}         Launch interactive Terminal User Interface"
echo -e "  ${BOLD}ldrago --help${NC}      Explore all commands, flags, and features"
echo -e "${GREEN}==========================================================${NC}"
echo ""
