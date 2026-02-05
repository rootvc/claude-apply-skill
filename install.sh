#!/bin/bash
# Root Ventures Apply Skill - One-line installer
# Supports both text mode (Claude CLI) and voice mode (native app)

set -e

SKILL_DIR="$HOME/.claude/skills/root-ventures-apply"
REPO_URL="https://raw.githubusercontent.com/rootvc/claude-apply-skill/main"

echo ""
echo "🚀 Root Ventures Apply"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  Choose your mode:"
echo ""
echo "  [1] 🎙️  Voice mode (recommended)"
echo "      Native app with speech recognition and text-to-speech"
echo ""
echo "  [2] 💬 Text mode"
echo "      Claude CLI chat interface"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

read -p "Enter choice (1 or 2): " choice </dev/tty

case $choice in
  1)
    # Voice mode - build and run VUI
    echo ""
    echo "🎙️  Starting voice mode..."
    echo ""

    # Check for Rust
    if ! command -v cargo &> /dev/null; then
      echo "📦 Rust not found. Installing via rustup..."
      echo ""
      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
      source "$HOME/.cargo/env"
      echo ""
      echo "✅ Rust installed successfully!"
      echo ""
    fi

    # Clone repo if needed, or use current dir if already in it
    if [ -f "vui/Cargo.toml" ]; then
      cd vui
    elif [ -f "Cargo.toml" ] && grep -q 'name = "vui"' Cargo.toml; then
      : # already in vui directory
    else
      TEMP_DIR=$(mktemp -d)
      echo "📥 Downloading voice app..."
      git clone --depth 1 https://github.com/rootvc/claude-apply-skill.git "$TEMP_DIR"
      cd "$TEMP_DIR/vui"
      echo ""
    fi

    echo "🔨 Building voice app (this may take a minute)..."
    echo ""
    cargo build --release

    echo ""
    echo "✅ Build complete!"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "  Launching voice app..."
    echo ""
    echo "  Just start talking when you see the circle!"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    sleep 1
    ./target/release/vui
    ;;

  2)
    # Text mode - install Claude CLI skill
    echo ""
    echo "💬 Installing text mode skill..."
    echo ""

    # Create skills directory if it doesn't exist
    mkdir -p "$SKILL_DIR"

    # Download files
    echo "📥 Downloading skill files..."
    curl -fsSL "$REPO_URL/skill.json" -o "$SKILL_DIR/skill.json"
    curl -fsSL "$REPO_URL/prompt.txt" -o "$SKILL_DIR/prompt.txt"
    curl -fsSL "$REPO_URL/apply.sh" -o "$SKILL_DIR/apply.sh"
    curl -fsSL "$REPO_URL/README.md" -o "$SKILL_DIR/README.md"

    # Make apply.sh executable
    chmod +x "$SKILL_DIR/apply.sh"

    echo ""
    echo "✅ Root Ventures Apply Skill installed successfully!"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "  Launching Claude..."
    echo ""
    echo "  Claude will confirm it's ready, then just say:"
    echo "  'I want to apply to Root Ventures'"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    # Check if Claude CLI is installed
    # Try multiple ways to find claude (handles aliases, PATH, and direct installation)
    CLAUDE_CMD=""

    if command -v claude &> /dev/null; then
      CLAUDE_CMD="claude"
    elif [ -f "$HOME/.claude/local/claude" ]; then
      CLAUDE_CMD="$HOME/.claude/local/claude"
    elif [ -f "/usr/local/bin/claude" ]; then
      CLAUDE_CMD="/usr/local/bin/claude"
    fi

    if [ -z "$CLAUDE_CMD" ]; then
      echo "⚠️  Claude CLI not found."
      echo ""
      echo "📦 Installing Claude CLI..."
      echo ""

      # Install Claude CLI using official installer
      echo "Installing Claude CLI..."
      curl -fsSL https://claude.ai/install.sh | bash

      # Add to PATH if not already there
      if ! command -v claude &> /dev/null; then
        export PATH="$HOME/.claude/local:$PATH"

        # Add to shell profile
        SHELL_PROFILE=""
        if [ -f "$HOME/.zshrc" ]; then
          SHELL_PROFILE="$HOME/.zshrc"
        elif [ -f "$HOME/.bashrc" ]; then
          SHELL_PROFILE="$HOME/.bashrc"
        elif [ -f "$HOME/.bash_profile" ]; then
          SHELL_PROFILE="$HOME/.bash_profile"
        fi

        if [ -n "$SHELL_PROFILE" ]; then
          if ! grep -q "/.claude/local" "$SHELL_PROFILE"; then
            echo 'export PATH="$HOME/.claude/local:$PATH"' >> "$SHELL_PROFILE"
            echo "✅ Added Claude CLI to PATH in $SHELL_PROFILE"
          fi
        fi
      fi

      # Verify installation
      if command -v claude &> /dev/null; then
        CLAUDE_CMD="claude"
        echo ""
        echo "✅ Claude CLI installed successfully!"
        echo ""
      elif [ -f "$HOME/.claude/local/claude" ]; then
        CLAUDE_CMD="$HOME/.claude/local/claude"
        echo ""
        echo "✅ Claude CLI installed successfully!"
        echo ""
      else
        echo ""
        echo "❌ Claude CLI installation failed."
        echo "Please install manually from: https://claude.ai/download"
        echo ""
        echo "After installing, run this to apply:"
        echo "  claude 'Read ~/.claude/skills/root-ventures-apply/prompt.txt then I want to apply'"
        echo ""
        exit 1
      fi
    fi

    # Launch Claude with the skill pre-loaded
    sleep 1
    exec "$CLAUDE_CMD" "Read ~/.claude/skills/root-ventures-apply/prompt.txt"
    ;;

  *)
    echo ""
    echo "❌ Invalid choice. Please run again and enter 1 or 2."
    echo ""
    exit 1
    ;;
esac
