# Root Ventures Application Skill

Apply to Root Ventures positions directly through Claude or Codex CLI.

## Installation & Usage

Just run this one command:

```bash
bash -c "$(curl -fsSL https://raw.githubusercontent.com/rootvc/claude-apply-skill/main/install.sh)"
```

The installer will:
- Detect whether you have Claude, Codex, or both
- Prompt you when a choice is needed
- Download the skill files
- Launch the selected CLI

Then simply say:
```
I want to apply to Root Ventures
```

## What You'll Provide

- Your name (required)
- Your email (required)
- LinkedIn profile (optional)
- GitHub username (optional)
- Why you're interested in Root (optional)

## How It Works

1. You run the one-line install command
2. The installer selects (or asks you to select) Claude or Codex
3. The installer downloads skill files and launches your CLI
4. Your CLI loads the skill instructions and waits for you
5. You say "I want to apply to Root Ventures"
6. The CLI starts the conversational application process
7. You provide your information naturally
8. The application script submits your application to Attio
9. You receive immediate confirmation

## Example

```bash
$ bash -c "$(curl -fsSL https://raw.githubusercontent.com/rootvc/claude-apply-skill/main/install.sh)"

🚀 Installing Root Ventures Apply Skill...
📥 Downloading skill files...
✅ Root Ventures Apply Skill installed successfully!

Launching Claude...

Once Claude opens, just say: 'I want to apply to Root Ventures'

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[Claude launches]

Claude: Root Ventures application skill installed. Claude can now help you apply for a job at Root Ventures

You: I want to apply to Root Ventures

Claude: Great! Root Ventures is looking for a technical associate in SF.
        Let me collect some information. What's your name?

You: Jane Doe

Claude: Thanks Jane! What's your email address?

        [... continues conversationally ...]

Claude: ✅ Application submitted successfully!
```

## Source Tracking

Applications submitted through this skill are tagged with `source: "Claude Skill"` or `source: "Codex Skill"` in Attio, and "Applied using Claude Skill" / "Applied using Codex Skill" is added to your notes.

## Other Ways to Apply

- **Web Terminal**: https://root.vc (type `apply 1`)
- **Email**: hello@root.vc

## About Root Ventures

Root Ventures is a San Francisco-based deep tech seed fund investing in bold engineers building the future.

- Website: https://root.vc
- Twitter: [@rootvc](https://twitter.com/rootvc)
