# Root Ventures Application Skill

Apply to Root Ventures positions directly through Claude CLI.

## Installation & Usage

Just run this one command:

```bash
curl -fsSL https://raw.githubusercontent.com/rootvc/claude-apply-skill/main/install.sh | bash
```

The installer will:
- Download the skill files
- Automatically launch Claude CLI
- Pre-load the application skill

Then simply say to Claude:
```
I want to apply to Root Ventures
```

Claude will then:
- Guide you through the application
- Collect your information conversationally
- Submit directly to Root Ventures

## What You'll Provide

- Your name (required)
- Your email (required)
- LinkedIn profile (optional)
- GitHub username (optional)
- Why you're interested in Root (optional)

## How It Works

1. You run the one-line install command
2. The installer downloads skill files and launches Claude
3. Claude loads the skill instructions and waits for you
4. You say "I want to apply to Root Ventures"
5. Claude starts the conversational application process
6. You provide your information naturally
7. Claude submits your application to Attio
8. You receive immediate confirmation

## Example

```bash
$ curl -fsSL https://raw.githubusercontent.com/rootvc/claude-apply-skill/main/install.sh | bash

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

Applications submitted through this skill are tagged with `source: "Claude Skill"` in Attio, and "Applied using claude skill" is added to your notes.

## Other Ways to Apply

- **Web Terminal**: https://root.vc (type `apply 1`)
- **Email**: hello@root.vc

## About Root Ventures

Root Ventures is a San Francisco-based deep tech seed fund investing in bold engineers building the future.

- Website: https://root.vc
- Twitter: [@rootvc](https://twitter.com/rootvc)
