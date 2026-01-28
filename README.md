# Root Ventures Application Skill

Apply to Root Ventures positions directly through Claude CLI.

## Usage

In Claude CLI, follow these steps:

**Step 1:** Load the skill by saying:
```
Read ~/.claude/skills/root-ventures-apply/prompt.txt
```

**Step 2:** Then say:
```
I want to apply to Root Ventures
```

Claude will help you through the application process by collecting:
- Your name (required)
- Your email (required)
- LinkedIn profile (optional)
- GitHub username (optional)
- Why Root? What makes you a great fit? (optional)

## How It Works

1. You tell Claude you want to apply to Root Ventures
2. Claude automatically detects this and begins collecting your information
3. Claude asks for your name, email, LinkedIn, GitHub, and why you're interested
4. Claude submits your application directly to Attio
5. You receive immediate confirmation

## Example Conversation

```
You: I want to apply to Root Ventures

Claude: Great! Root Ventures is looking for a technical associate in SF.
        What's your name and email?

You: Jane Doe, jane@example.com, GitHub is janedoe,
     I'm excited about deep tech investing

Claude: [Submits your application]
        ✅ Application submitted successfully!
```

## Manual Usage

You can also invoke the skill directly:

```bash
~/.claude/skills/root-ventures-apply/apply.sh \
  --name "Your Name" \
  --email "your@email.com" \
  --linkedin "https://linkedin.com/in/yourprofile" \
  --github "yourgithub" \
  --notes "Why you're interested in Root"
```

## Source Tracking

Applications submitted through this skill are tagged with `source: "Claude Skill"` in Attio.

## Other Ways to Apply

- **Web Terminal**: https://root.vc (type `apply 1`)
- **Email**: hello@root.vc

## About Root Ventures

Root Ventures is a San Francisco-based deep tech seed fund investing in bold engineers building the future.

- Website: https://root.vc
- Twitter: [@rootvc](https://twitter.com/rootvc)
