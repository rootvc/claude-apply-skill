#!/bin/bash
# Root Ventures Apply Skill - One-line installer for Claude CLI

set -e

SKILL_DIR="$HOME/.claude/skills/root-ventures-apply"
REPO_URL="https://raw.githubusercontent.com/rootvc/claude-apply-skill/main"

echo ""
echo "🚀 Installing Root Ventures Apply Skill..."
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
echo "Usage:"
echo "  1. Open Claude Code CLI (type 'claude' in your terminal)"
echo "  2. Say: 'I want to apply to Root Ventures'"
echo "  3. Claude will guide you through the application"
echo ""
echo "Learn more:"
echo "  https://root.vc"
echo ""
