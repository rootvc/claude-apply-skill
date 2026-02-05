# Root Ventures Application

Apply to Root Ventures directly through voice or text conversation with Claude.

## Quick Start

```bash
curl -fsSL https://raw.githubusercontent.com/rootvc/claude-apply-skill/main/install.sh | bash
```

You'll be prompted to choose:

1. **Voice mode** (recommended) — Native app with speech recognition and text-to-speech
2. **Text mode** — Claude CLI chat interface

## Voice Mode

A native macOS app that lets you have a spoken conversation with Claude. Just talk naturally — Claude will interview you, fill out the application form in real-time, and submit when you're ready.

**Requirements:**
- macOS
- `ANTHROPIC_API_KEY` and `ELEVENLABS_API_KEY` in environment or `.env`

The installer will install Rust if needed and build the app (~1 min first time).

**Features:**
- Natural voice conversation
- Barge-in support — interrupt Claude mid-sentence
- Live form updates as you speak
- Automatic submission to Attio CRM

## Text Mode

Uses Claude CLI for a text-based chat. The installer downloads the skill files and launches Claude automatically.

Then just say:
```
I want to apply to Root Ventures
```

## What You'll Provide

- Name (required)
- Email (required)
- LinkedIn profile
- GitHub username
- Why you're interested (collected through interview questions)

## Other Ways to Apply

- **Web Terminal**: https://root.vc (type `apply 1`)
- **Email**: hello@root.vc

## About Root Ventures

Root Ventures is a San Francisco-based deep tech seed fund investing in bold engineers building the future.

- Website: https://root.vc
- Twitter: [@rootvc](https://twitter.com/rootvc)
