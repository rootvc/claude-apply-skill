#!/bin/bash
# Root Ventures Apply Skill - Unified installer for Claude or Codex CLI

set -e

SKILL_NAME="root-ventures-apply"
REPO_URL="https://raw.githubusercontent.com/rootvc/claude-apply-skill/main"

CLAUDE_CMD=""
CODEX_CMD=""
TARGET=""

# Detect Claude CLI
if command -v claude &> /dev/null; then
  CLAUDE_CMD="claude"
elif [ -f "$HOME/.claude/local/claude" ]; then
  CLAUDE_CMD="$HOME/.claude/local/claude"
elif [ -f "/usr/local/bin/claude" ]; then
  CLAUDE_CMD="/usr/local/bin/claude"
fi

# Detect Codex CLI
if command -v codex &> /dev/null; then
  CODEX_CMD="codex"
fi

prompt_choice() {
  local prompt="$1"
  local choice=""

  if [[ ! -t 0 && ! -r /dev/tty ]]; then
    echo ""
    echo "No interactive terminal detected."
    echo "Re-run this installer in a terminal without piping stdin."
    exit 1
  fi

  while true; do
    echo ""
    echo "$prompt"
    echo "1) Claude"
    echo "2) Codex"
    if [[ -t 0 ]]; then
      read -r choice
    else
      read -r choice < /dev/tty
    fi
    case "$choice" in
      1|claude|Claude|CLAUDE)
        TARGET="claude"
        return
        ;;
      2|codex|Codex|CODEX)
        TARGET="codex"
        return
        ;;
      *)
        echo "Please choose 1 or 2."
        ;;
    esac
  done
}

if [[ -n "$CLAUDE_CMD" && -n "$CODEX_CMD" ]]; then
  prompt_choice "Both Claude and Codex are installed. Which would you like to use?"
elif [[ -n "$CLAUDE_CMD" ]]; then
  TARGET="claude"
elif [[ -n "$CODEX_CMD" ]]; then
  TARGET="codex"
else
  prompt_choice "Neither Claude nor Codex CLI was found. Which should I install?"
fi

if [[ "$TARGET" == "claude" ]]; then
  SKILL_DIR="$HOME/.claude/skills/$SKILL_NAME"

  echo ""
  echo "🚀 Installing Root Ventures Apply Skill for Claude..."
  echo ""

  mkdir -p "$SKILL_DIR"

  echo "📥 Downloading skill files..."
  curl -fsSL "$REPO_URL/skill.json" -o "$SKILL_DIR/skill.json"
  curl -fsSL "$REPO_URL/prompt.txt" -o "$SKILL_DIR/prompt.txt"
  curl -fsSL "$REPO_URL/apply.sh" -o "$SKILL_DIR/apply.sh"
  curl -fsSL "$REPO_URL/README.md" -o "$SKILL_DIR/README.md"

  chmod +x "$SKILL_DIR/apply.sh"

  if [[ -z "$CLAUDE_CMD" ]]; then
    echo ""
    echo "⚠️  Claude CLI not found."
    echo ""
    echo "📦 Installing Claude CLI..."
    echo ""

    curl -fsSL https://claude.ai/install.sh | bash

    if ! command -v claude &> /dev/null; then
      export PATH="$HOME/.claude/local:$PATH"

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

  sleep 1
  exec "$CLAUDE_CMD" "Read ~/.claude/skills/root-ventures-apply/prompt.txt"
fi

if [[ "$TARGET" == "codex" ]]; then
  if [ -d "$HOME/.codex" ]; then
    SKILLS_BASE="$HOME/.codex/skills"
  elif [ -d "$HOME/.agents" ]; then
    SKILLS_BASE="$HOME/.agents/skills"
  else
    SKILLS_BASE="$HOME/.codex/skills"
  fi

  SKILL_DIR="$SKILLS_BASE/$SKILL_NAME"
  SKILL_JSON="$SKILL_DIR/skill.json"
  PROMPT_FILE="$SKILL_DIR/prompt.txt"

  echo ""
  echo "🚀 Installing Root Ventures Apply Skill for Codex..."
  echo ""

  mkdir -p "$SKILL_DIR"

  echo "📥 Downloading skill files..."
  curl -fsSL "$REPO_URL/skill.json" -o "$SKILL_JSON"
  curl -fsSL "$REPO_URL/prompt.txt" -o "$PROMPT_FILE"
  curl -fsSL "$REPO_URL/apply.sh" -o "$SKILL_DIR/apply.sh"

  SKILL_BODY_FILE=$(mktemp)
  PROMPT_HEAD=$(head -n 1 "$PROMPT_FILE")

  awk 'BEGIN{in_section=0}
    /^## When to Activate/{in_section=1}
    /^## Important: Initial Response/{if(in_section){exit}}
    {if(in_section) print}
  ' "$PROMPT_FILE" > "$SKILL_BODY_FILE"

  if [ ! -s "$SKILL_BODY_FILE" ]; then
    cp "$PROMPT_FILE" "$SKILL_BODY_FILE"
  fi

  if command -v jq &> /dev/null; then
    SKILL_NAME_VAL=$(jq -r '.name // "root-ventures-apply"' "$SKILL_JSON")
    SKILL_DESC_VAL=$(jq -r '.description // "Apply to Root Ventures"' "$SKILL_JSON")
  else
    SKILL_NAME_VAL="root-ventures-apply"
    SKILL_DESC_VAL="Apply to Root Ventures"
  fi

  {
    echo "---"
    echo "name: \"$SKILL_NAME_VAL\""
    echo "description: \"$SKILL_DESC_VAL\""
    echo "metadata:"
    echo "  short-description: \"$SKILL_DESC_VAL\""
    echo "---"
    echo ""
    if [ -n "$PROMPT_HEAD" ]; then
      printf '%s\n\n' "$PROMPT_HEAD"
    fi
    cat "$SKILL_BODY_FILE"
  } > "$SKILL_DIR/SKILL.md"

  rm -f "$SKILL_BODY_FILE"

  chmod +x "$SKILL_DIR/apply.sh"

  if [[ -z "$CODEX_CMD" ]]; then
    echo ""
    echo "⚠️  Codex CLI not found."
    echo ""
    echo "📦 Installing Codex CLI..."
    echo ""

    if command -v npm &> /dev/null; then
      npm i -g @openai/codex
    else
      echo "❌ npm not found."
      echo "Install Node.js, then run: npm i -g @openai/codex"
      exit 1
    fi

    if command -v codex &> /dev/null; then
      CODEX_CMD="codex"
      echo ""
      echo "✅ Codex CLI installed successfully!"
      echo ""
    else
      echo ""
      echo "❌ Codex CLI installation failed."
      echo "Please install manually, then run: codex"
      echo "Docs: https://developers.openai.com/codex/cli"
      echo ""
      exit 1
    fi
  fi

  echo ""
  echo "✅ Root Ventures Apply Skill installed successfully!"
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""
  echo "  Launching Codex..."
  echo ""
  echo "  If Codex was already running, restart it to load the new skill."
  echo "  Then say: 'I want to apply to Root Ventures'"
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""

  sleep 1
  if [[ -t 0 ]]; then
    exec "$CODEX_CMD"
  elif [[ -r /dev/tty ]]; then
    exec "$CODEX_CMD" < /dev/tty
  else
    echo "❌ Codex requires an interactive terminal."
    echo "Re-run this installer in a terminal without piping stdin."
    exit 1
  fi
fi
