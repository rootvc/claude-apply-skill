# Root Ventures Application Skill

Apply to Root Ventures positions directly through Claude CLI.

## Usage

In Claude CLI, simply say:

```
I want to apply to Root Ventures
```

or

```
Apply for the Root Ventures associate position
```

Claude will help you through the application process by collecting:
- Your name (required)
- Your email (required)
- LinkedIn profile (optional)
- GitHub username (optional)
- Why Root? What makes you a great fit? (optional)

## How It Works

1. You start a conversation with Claude about applying
2. Claude collects your information naturally through conversation
3. Claude calls this skill with the collected information
4. The skill submits your application directly to Attio
5. You receive confirmation of submission

## Example Conversation

```
You: I'd like to apply to Root Ventures. My name is Jane Doe,
     email is jane@example.com, GitHub is janedoe, and I'm
     excited about deep tech investing because...

Claude: [Uses the skill to submit your application]
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
