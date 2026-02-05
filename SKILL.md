# Claude Skill Development Guide

This document explains how the Root Ventures Apply skill is structured and how to create similar Claude skills.

## What is a Claude Skill?

A Claude skill is a package of files that extends Claude CLI with new capabilities. Skills are installed to `~/.claude/skills/` and are automatically loaded when Claude starts.

## Skill Structure

```
~/.claude/skills/root-ventures-apply/
├── skill.json          # Skill metadata and configuration
├── prompt.txt          # Instructions for Claude on how to use the skill
├── apply.sh            # Executable script that performs the main action
└── install.sh          # (optional) Installation script
```

### skill.json

The metadata file that defines the skill:

```json
{
  "name": "root-ventures-apply",
  "version": "1.0.0",
  "description": "Apply to Root Ventures positions directly through Claude",
  "entrypoint": "apply.sh",
  "author": "Root Ventures",
  "triggers": [
    "apply to root ventures",
    "root ventures application",
    "apply for root ventures job"
  ]
}
```

**Fields:**
- `name`: Unique identifier for the skill (kebab-case)
- `version`: Semantic version number
- `description`: Brief description of what the skill does
- `entrypoint`: The main script file that executes the skill's action
- `author`: Creator of the skill
- `triggers`: Array of phrases that should activate this skill

### prompt.txt

Instructions that tell Claude how to use the skill. This file is read by Claude when the skill is loaded.

**Key sections:**

1. **Activation Detection**: When and how to trigger the skill
2. **User Interaction Flow**: How to collect information from users
3. **Execution Instructions**: How to call the entrypoint script
4. **Response Formatting**: How to present results to users

**Best Practices:**

- Be explicit about when to activate
- Provide clear examples of user interactions
- Specify exact command formats for script invocation
- Include conversational guidelines (tone, pacing, emoji usage)
- Give examples of complete flows

### Entrypoint Script (apply.sh)

The executable that performs the skill's main action. Must be:
- Executable (`chmod +x`)
- Self-contained or with clear dependencies
- Return clear success/failure messages
- Handle errors gracefully

**This skill's apply.sh:**
- Accepts command-line arguments (`--name`, `--email`, etc.)
- Makes API call to Attio CRM
- Returns formatted success/failure message
- Includes source tracking for analytics

## How Skills Work

1. **Installation**: Files are copied to `~/.claude/skills/[skill-name]/`
2. **Loading**: When Claude CLI starts, it reads all `prompt.txt` files
3. **Activation**: Claude detects trigger phrases and follows prompt instructions
4. **Execution**: Claude invokes the entrypoint script via Bash tool
5. **Response**: Claude formats the script output for the user

## Creating Your Own Skill

### Step 1: Define the Use Case

Determine:
- What action should the skill perform?
- What information needs to be collected?
- What external systems will it interact with?
- How should it handle errors?

### Step 2: Create skill.json

```json
{
  "name": "your-skill-name",
  "version": "1.0.0",
  "description": "Brief description",
  "entrypoint": "action.sh",
  "author": "Your Name",
  "triggers": [
    "trigger phrase 1",
    "trigger phrase 2"
  ]
}
```

### Step 3: Write prompt.txt

Structure it as:
```
You have access to [skill name] installed on this computer.

## When to Activate

Detect when the user expresses [specific intent]:
- "trigger phrase 1"
- "trigger phrase 2"
- [variations]

## How to Handle [Action]

1. [Step-by-step instructions for Claude]
2. [Information to collect]
3. [How to invoke the script]
4. [How to format responses]

## Example Flow

User: "trigger phrase"
You: [Expected response]
```

### Step 4: Build the Entrypoint Script

```bash
#!/bin/bash

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --arg1)
      ARG1="$2"
      shift 2
      ;;
    --arg2)
      ARG2="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Validate inputs
if [ -z "$ARG1" ]; then
  echo "Error: --arg1 is required"
  exit 1
fi

# Perform action
# [Your logic here]

# Return formatted output
echo "✅ Action completed successfully!"
```

### Step 5: Create install.sh (Optional)

```bash
#!/bin/bash

SKILL_NAME="your-skill-name"
SKILL_DIR="$HOME/.claude/skills/$SKILL_NAME"

echo "🚀 Installing $SKILL_NAME..."

# Create directory
mkdir -p "$SKILL_DIR"

# Download files
curl -fsSL "https://raw.githubusercontent.com/you/repo/main/skill.json" -o "$SKILL_DIR/skill.json"
curl -fsSL "https://raw.githubusercontent.com/you/repo/main/prompt.txt" -o "$SKILL_DIR/prompt.txt"
curl -fsSL "https://raw.githubusercontent.com/you/repo/main/action.sh" -o "$SKILL_DIR/action.sh"

# Make executable
chmod +x "$SKILL_DIR/action.sh"

echo "✅ $SKILL_NAME installed successfully!"
```

## Testing Your Skill

### Local Testing

1. **Install locally:**
   ```bash
   mkdir -p ~/.claude/skills/your-skill-name
   cp * ~/.claude/skills/your-skill-name/
   chmod +x ~/.claude/skills/your-skill-name/*.sh
   ```

2. **Test the script directly:**
   ```bash
   ~/.claude/skills/your-skill-name/action.sh --arg1 "test" --arg2 "test"
   ```

3. **Test in Claude CLI:**
   - Launch Claude: `claude`
   - Use a trigger phrase
   - Verify Claude follows the prompt.txt instructions
   - Check that script execution works correctly
   - Verify output formatting

### Integration Testing

1. Test all conversation flows
2. Test error cases (missing data, API failures)
3. Test edge cases (special characters, very long inputs)
4. Test on fresh installation

## Advanced Features

### Environment Variables

Store sensitive data in environment variables:

```bash
# In apply.sh
API_KEY="${ATTIO_API_KEY}"
if [ -z "$API_KEY" ]; then
  echo "Error: ATTIO_API_KEY not set"
  exit 1
fi
```

Users set them in `~/.bashrc` or `~/.zshrc`:
```bash
export ATTIO_API_KEY="your-key-here"
```

### Multi-Step Workflows

For complex skills, prompt.txt can guide Claude through multiple script invocations:

```
1. First, run: ~/.claude/skills/your-skill/validate.sh
2. If validation passes, run: ~/.claude/skills/your-skill/execute.sh
3. Finally, run: ~/.claude/skills/your-skill/confirm.sh
```

### Configuration Files

Create `~/.claude/skills/your-skill/config.json` for user preferences:

```json
{
  "api_endpoint": "https://api.example.com",
  "default_options": {
    "verbose": true
  }
}
```

Read in your script:
```bash
CONFIG_FILE="$HOME/.claude/skills/your-skill/config.json"
ENDPOINT=$(jq -r '.api_endpoint' "$CONFIG_FILE")
```

## Security Considerations

1. **Never hardcode secrets** - Use environment variables
2. **Validate all inputs** - Sanitize user-provided data
3. **Use HTTPS** - For all API calls
4. **Minimal permissions** - Only request what's needed
5. **Clear error messages** - Don't expose sensitive details
6. **Audit logging** - Track when/how the skill is used

## Distribution

See [DISTRIBUTION.md](DISTRIBUTION.md) for details on:
- Publishing to GitHub
- Creating one-line installers
- Marketing the skill
- Analytics and tracking

## Skill Ideas

Potential skills you could create:
- Job application workflows (like this one)
- CRM integrations (create leads, update contacts)
- Project management (create tasks in Jira/Linear)
- Code deployment (push to production with confirmation)
- Data collection (surveys, feedback forms)
- Notification systems (send alerts via Slack/Discord)
- Calendar scheduling (book meetings)
- Document generation (create PDFs, reports)

## Resources

- **Claude CLI Documentation**: https://docs.anthropic.com/claude/cli
- **Example Skills**: https://github.com/anthropics/claude-skills
- **Claude API**: https://docs.anthropic.com/claude/reference

## Support

For questions about this skill:
- GitHub Issues: https://github.com/rootvc/claude-apply-skill/issues
- Email: hello@root.vc

For questions about Claude CLI:
- Claude Documentation: https://docs.anthropic.com/claude/cli
- Discord: https://discord.gg/anthropic

---

## This Skill's Architecture

### Components

**skill.json**
- Defines "root-ventures-apply" as the skill name
- Lists trigger phrases related to applying
- Sets apply.sh as the entrypoint

**prompt.txt**
- Detects application intent from user messages
- Guides Claude through conversational information collection
- Specifies exactly how to call apply.sh with collected data
- Provides tone guidance (casual, not too many exclamation points)

**apply.sh**
- Parses command-line arguments (name, email, linkedin, github, notes)
- Validates required fields (name, email)
- Makes authenticated API call to Attio CRM
- Tags application with "source: Claude Skill"
- Returns formatted success/error message

**install.sh**
- Downloads all necessary files from GitHub
- Creates skill directory structure
- Sets correct permissions
- Launches Claude CLI automatically with skill pre-loaded

### Data Flow

```
User: "I want to apply to Root Ventures"
  ↓
Claude detects trigger (from prompt.txt)
  ↓
Claude collects information conversationally
  ↓
Claude calls: ~/.claude/skills/root-ventures-apply/apply.sh \
  --name "Jane" --email "jane@ex.com" --github "jane" \
  --notes "Interested in deep tech..."
  ↓
apply.sh makes API call to Attio
  ↓
Attio creates/updates person record
  ↓
apply.sh returns success message
  ↓
Claude formats and shows message to user
```

### Customization Points

To adapt this skill for another company:

1. Update skill.json:
   - Change `name` to "company-name-apply"
   - Update `description` and `author`
   - Modify `triggers` to match company name

2. Update prompt.txt:
   - Change company name and position details
   - Adjust information collection requirements
   - Modify tone/style to match company culture

3. Update apply.sh:
   - Replace Attio API endpoint with your CRM/database
   - Update authentication method
   - Modify data fields to match your system
   - Customize success messages

4. Update install.sh:
   - Change GitHub URLs to your repository
   - Update skill name references

5. Test thoroughly with your backend systems
