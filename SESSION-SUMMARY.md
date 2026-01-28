# Root Ventures Claude Apply Skill - Development Session Summary

**Date:** January 27, 2026
**Goal:** Create a Claude skill for job applications that's as seamless as possible
**Repository:** https://github.com/rootvc/claude-apply-skill

---

## 🎯 Final Solution

### One-Line Installation & Application
```bash
curl -fsSL https://raw.githubusercontent.com/rootvc/claude-apply-skill/main/install.sh | bash
```

This single command:
1. Downloads skill files to `~/.claude/skills/root-ventures-apply/`
2. Automatically launches Claude CLI
3. Pre-loads the application skill
4. Starts the conversational application process immediately

### User Experience Flow

```
User runs: curl -fsSL https://raw.githubusercontent.com/rootvc/claude-apply-skill/main/install.sh | bash

🚀 Installing Root Ventures Apply Skill...
📥 Downloading skill files...
✅ Root Ventures Apply Skill installed successfully!

Starting Claude with the application skill loaded...

[Claude launches]

Claude: Great! Root Ventures is looking for a technical associate in SF...
        Let me collect some information. What's your name?

User: Jane Doe

Claude: Thanks Jane! What's your email address?

[... conversational collection continues ...]

Claude: ✅ Application submitted successfully!
```

---

## 📁 Repository Structure

```
claude-apply-skill/
├── skill.json              # Skill metadata
├── prompt.txt              # Instructions for Claude on how to handle applications
├── apply.sh                # Bash script that submits to Attio webhook
├── install.sh              # One-line installer with auto-launch
├── README.md               # User-facing documentation
├── DISTRIBUTION.md         # Marketing/distribution guide
├── website-snippet.html    # HTML for root.vc integration
└── SESSION-SUMMARY.md      # This file
```

---

## 🔧 Technical Implementation

### Key Files

**skill.json**
- Defines the skill metadata
- Name: `root-ventures-apply`
- Triggers: Various application-related phrases (for metadata only)

**prompt.txt**
- Loaded by Claude to understand how to handle applications
- Instructs Claude to:
  - Detect application intent
  - Collect info conversationally (name, email, LinkedIn, GitHub, notes)
  - Call apply.sh via Bash tool
  - Present all fields but accept graceful declines

**apply.sh**
- Bash script that:
  - Validates required fields (name, email)
  - Appends "Applied using claude skill" to notes
  - Uses jq to properly escape JSON (handles multiline notes)
  - POSTs to Attio webhook
  - Returns formatted success/error message

**install.sh**
- Downloads all skill files from GitHub
- Checks if Claude CLI is installed
- Automatically launches: `claude "Read ~/.claude/skills/root-ventures-apply/prompt.txt then I want to apply"`
- Graceful fallback if Claude not found

---

## 🔑 Key Technical Decisions

### 1. JSON Escaping with jq
**Problem:** Multiline notes were breaking JSON payload, causing 500 errors from Attio

**Solution:** Use jq to build JSON properly
```bash
JSON_PAYLOAD=$(jq -n \
  --arg name "$NAME" \
  --arg email "$EMAIL" \
  --arg notes "$NOTES" \
  '{name: $name, email: $email, notes: $notes, ...}')
```

### 2. Automatic Skill Attribution
Every application automatically gets:
- `source: "Claude Skill"` field
- `notes` field appended with: "Applied using claude skill"

### 3. Auto-Launch in Installer
Uses `exec claude "..."` to replace the installer process with Claude, automatically loading the skill

### 4. Conversational Detection
Claude detects phrases like "I want to apply to Root Ventures" by reading prompt.txt which instructs it to immediately begin the application flow

---

## 🌐 Attio Integration

**Webhook URL:**
```
https://hooks.attio.com/w/1d456d59-a7ac-4211-ac1d-fac612f7f491/5fc14931-0124-4121-b281-1dbfb64dceb2
```

**Payload Format:**
```json
{
  "name": "Applicant Name",
  "email": "email@example.com",
  "linkedin": "linkedin.com/in/profile",
  "github": "username",
  "notes": "Why they're interested\n\nApplied using claude skill",
  "position": "Venture Capital Associate",
  "source": "Claude Skill"
}
```

---

## 📊 Evolution of the Solution

### Initial Approach
- Simple skill with prompt.txt
- Users had to manually invoke with `/root-ventures-apply` slash command

### Iteration 1: Natural Language
- Added conversational detection
- Users say "I want to apply to Root Ventures"
- Issue: Skill triggers didn't work automatically

### Iteration 2: Two-Step Process
- Step 1: Load prompt.txt
- Step 2: Say "I want to apply"
- Still required manual paste

### Iteration 3: Standalone Bash Wrapper
- Created `apply-to-root` command
- Pure bash interaction, no Claude needed
- Issue: No longer a "Claude skill"

### Final Solution: Auto-Launch
- One-line install that automatically launches Claude
- Pre-loads the skill with the prompt
- True Claude skill with seamless UX
- Best of both worlds!

---

## 🚀 Distribution Strategy

### Primary Channel: One-Liner
Share this everywhere:
```bash
curl -fsSL https://raw.githubusercontent.com/rootvc/claude-apply-skill/main/install.sh | bash
```

### Where to Share
- Job postings on root.vc
- Email signatures
- Twitter/X posts
- LinkedIn posts
- GitHub repositories
- Direct messages to candidates

### Marketing Copy

**Short version:**
"Apply via AI: Just run this command and Claude handles the rest"

**Medium version:**
"We're one of the first VCs to accept applications through Claude AI. Install our skill with one command and let AI guide you through the application."

**Long version:**
See DISTRIBUTION.md for full marketing templates

---

## 🔍 Troubleshooting

### Common Issues

**"Command not found: claude"**
- User needs to install Claude CLI from https://claude.ai/download
- Installer provides fallback instructions

**"Skill not appearing"**
- Restart Claude CLI
- Verify files installed: `ls ~/.claude/skills/root-ventures-apply/`

**"Unknown skill" error**
- Early versions had this - fixed by using Bash tool instead of Skill tool
- Make sure running latest version from GitHub

**Attio 500 error**
- Fixed by using jq for JSON escaping
- Was caused by multiline notes breaking JSON

---

## 📈 Tracking & Analytics

### How to Identify Claude Skill Applications in Attio

1. **Source Field:** `"source": "Claude Skill"`
2. **Notes Field:** Always ends with "Applied using claude skill"

### Metrics to Track
- Total applications via Claude skill
- Conversion rate vs other channels
- Time to complete application
- Drop-off points (if any)

---

## 🔐 Security Considerations

### Public Webhook URL
- The Attio webhook URL is public in the GitHub repo
- This is acceptable for job applications (write-only endpoint)
- Attio provides rate limiting
- No sensitive data exposure

### Alternative: Private Webhook
If you want to keep the webhook private:
1. Deploy a Netlify function at root.vc
2. Update apply.sh to POST to `https://root.vc/.netlify/functions/submit-application`
3. Set ATTIO_WEBHOOK_URL as environment variable in Netlify

---

## 🎨 Customization Guide

### Change Job Title/Description
Edit `prompt.txt`:
```
Position: Your Position Name
Location: Your Location
Company: Your Company (your description)
```

### Change Required/Optional Fields
Edit `prompt.txt` section:
```
2. **Collect information naturally through conversation:**
   - Name (required)
   - Email (required)
   - [Add/remove fields as needed]
```

### Change Attio Webhook
Edit `apply.sh` line 6:
```bash
ATTIO_WEBHOOK="https://hooks.attio.com/w/your-webhook-id"
```

### Customize Success Message
Edit `apply.sh` lines 99-108:
```bash
echo "✅ **Application submitted successfully!**"
echo "Your custom message here"
```

---

## 📝 Git History

Key commits:
- `f395a54` - Initial commit: Root Ventures Claude apply skill
- `afb2ff1` - Append 'Applied using claude skill' to notes
- `b67150d` - Fix JSON escaping with jq
- `035a633` - Simplify to conversational detection
- `d28ef6f` - Make installation fully automatic with auto-launch

---

## ✅ Final Checklist

- [x] Skill installs correctly
- [x] Auto-launches Claude
- [x] Conversational collection works
- [x] All fields prompted (name, email, LinkedIn, GitHub, notes)
- [x] Graceful handling of optional fields
- [x] Proper JSON escaping (multiline notes work)
- [x] Submits to Attio successfully
- [x] Attribution tags added automatically
- [x] Error handling for missing Claude CLI
- [x] Documentation complete
- [x] Marketing materials ready
- [x] Repository public and accessible

---

## 🔮 Future Enhancements

### Potential Improvements
1. Support multiple positions (allow user to choose)
2. Add resume upload capability
3. Integration with other job boards
4. Analytics dashboard for tracking applications
5. Email confirmation to applicant
6. Slack notification to Root team
7. Support for other CRMs besides Attio

### Alternative Channels
1. Web form at root.vc (for non-technical users)
2. API endpoint for programmatic applications
3. Mobile app integration
4. Zapier integration

---

## 📞 Support

For issues or questions:
- GitHub Issues: https://github.com/rootvc/claude-apply-skill/issues
- Email: hello@root.vc
- Twitter: [@rootvc](https://twitter.com/rootvc)

---

## 🎉 Success Metrics

This skill represents:
- **First-of-its-kind** VC application system via AI
- **One command** complete installation and usage
- **Zero friction** for technical candidates
- **Instant submission** to your CRM
- **Automatic tracking** of skill-based applications

---

**Session completed: January 27, 2026**
**Status: Production ready ✅**
**Repository: https://github.com/rootvc/claude-apply-skill**
