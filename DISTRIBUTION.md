# How to Distribute the Root Ventures Apply Skill

## Step 1: Upload to GitHub

1. Create a new public GitHub repository named `claude-apply-skill`
2. Upload these files:
   - `skill.json`
   - `prompt.txt`
   - `apply.sh`
   - `README.md`
   - `install.sh`

## Step 2: Give Applicants This One-Line Command

Once uploaded to GitHub, applicants can install with:

```bash
curl -fsSL https://raw.githubusercontent.com/rootvc/claude-apply-skill/main/install.sh | bash
```

**Note:** Replace `rootvc` with your actual GitHub username/organization

## Step 3: Add to Job Postings

### Website Copy

```markdown
## 🤖 Apply with Claude

We're one of the first VCs to accept applications through AI.

**If you have Claude CLI installed:**

```bash
curl -fsSL https://raw.githubusercontent.com/rootvc/claude-apply-skill/main/install.sh | bash
```

Then open Claude CLI and type: /root-ventures-apply

**Don't have Claude CLI?** Install from https://claude.ai/download
```

### Email Template

```
Subject: Apply to Root Ventures via Claude CLI

Hi [Name],

We've built a Claude skill that lets you apply directly through your terminal.

Installation (one command):
curl -fsSL https://raw.githubusercontent.com/rootvc/claude-apply-skill/main/install.sh | bash

Then open Claude CLI and say: "I want to apply to Root Ventures"

It's that simple. Looking forward to your application!

- Root Ventures Team
```

### Twitter/X Post

```
We're now accepting applications through @AnthropicAI Claude CLI 🤖

Engineers can apply by running one terminal command, then chatting with Claude.

Installation:
[link to GitHub]

This is what "technical recruiting" should look like in 2025.

#AI #VentureCapital #DeepTech
```

## Testing the Installation

Test locally before distributing:

```bash
# Clean install test
rm -rf ~/.claude/skills/root-ventures-apply
bash install.sh
```

Then open Claude CLI and type: /root-ventures-apply

## Alternative Distribution Methods

### 1. Host on Your Website

Host `install.sh` at `https://root.vc/claude-skill/install.sh`

One-liner becomes:
```bash
curl -fsSL https://root.vc/claude-skill/install.sh | bash
```

### 2. Direct Download

Create a download page where users can:
1. Download `root-ventures-apply-skill.zip`
2. Unzip to `~/.claude/skills/`
3. Run `chmod +x ~/.claude/skills/root-ventures-apply/apply.sh`

### 3. Include in Job Listing Emails

Send directly to candidates with installation instructions in the email.

## Analytics

Track installations by:
1. Adding a silent ping to install.sh (optional)
2. Check Attio for applications with `source: "Claude Skill"`
3. Monitor GitHub download stats

## Support

Common issues:
- **"Command not found: claude"** → User needs to install Claude CLI first
- **"Permission denied"** → Need to run `chmod +x apply.sh`
- **"Skill not appearing"** → Restart Claude CLI after installation
