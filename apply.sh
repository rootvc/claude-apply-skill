#!/bin/bash

# Root Ventures Job Application Skill
# Collects applicant information and submits to Attio

ATTIO_WEBHOOK="https://hooks.attio.com/w/1d456d59-a7ac-4211-ac1d-fac612f7f491/5fc14931-0124-4121-b281-1dbfb64dceb2"

echo "# Root Ventures Application"
echo ""
echo "**Venture Capital Associate** - San Francisco"
echo ""
echo "Root Ventures is looking for a technical associate to join our team."
echo "We're a deep tech seed fund that invests in bold engineers building the future."
echo ""
echo "What we're looking for:"
echo "• Strong technical background"
echo "• Genuine curiosity about emerging technologies"
echo "• Excellent communication skills"
echo "• Hustle and resourcefulness"
echo ""
echo "---"
echo ""
echo "Let's get your application started!"
echo ""

# Get applicant information from Claude context
# Claude will extract this from the conversation

# Parse arguments passed by Claude
NAME=""
EMAIL=""
LINKEDIN=""
GITHUB=""
NOTES=""
SOURCE_LABEL=""

# Simple argument parsing
while [[ $# -gt 0 ]]; do
  case $1 in
    --name)
      NAME="$2"
      shift 2
      ;;
    --email)
      EMAIL="$2"
      shift 2
      ;;
    --linkedin)
      LINKEDIN="$2"
      shift 2
      ;;
    --github)
      GITHUB="$2"
      shift 2
      ;;
    --notes)
      NOTES="$2"
      shift 2
      ;;
    --source)
      SOURCE_LABEL="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

# Default source label (keeps Claude behavior if not provided)
if [[ -z "$SOURCE_LABEL" ]]; then
  SOURCE_LABEL="Claude Skill"
fi

# Validate required fields
if [[ -z "$NAME" ]] || [[ -z "$EMAIL" ]]; then
  echo "❌ Error: Name and email are required."
  echo ""
  echo "Please provide:"
  echo "• Your full name"
  echo "• Your email address"
  echo "• LinkedIn profile (optional)"
  echo "• GitHub username (optional)"
  echo "• Why Root? What makes you a great fit? (optional)"
  exit 1
fi

# Append skill attribution to notes
if [[ -n "$NOTES" ]]; then
  NOTES="$NOTES

Applied using $SOURCE_LABEL"
else
  NOTES="Applied using $SOURCE_LABEL"
fi

# Build JSON payload using jq to properly escape values
JSON_PAYLOAD=$(jq -n \
  --arg name "$NAME" \
  --arg email "$EMAIL" \
  --arg linkedin "$LINKEDIN" \
  --arg github "$GITHUB" \
  --arg notes "$NOTES" \
  --arg source "$SOURCE_LABEL" \
  '{
    name: $name,
    email: $email,
    linkedin: $linkedin,
    github: $github,
    notes: $notes,
    position: "Venture Capital Associate",
    source: $source
  }'
)

# Submit to Attio
# Create secure temporary file
TEMP_FILE=$(mktemp)
trap "rm -f $TEMP_FILE" EXIT

HTTP_CODE=$(curl -s -w "%{http_code}" -o "$TEMP_FILE" \
  -X POST "$ATTIO_WEBHOOK" \
  -H "Content-Type: application/json" \
  -d "$JSON_PAYLOAD")

if [[ "$HTTP_CODE" == "202" ]] || [[ "$HTTP_CODE" == "200" ]]; then
  echo "✅ **Application submitted successfully!**"
  echo ""
  echo "Thank you for applying, $NAME!"
  echo ""
  echo "**What happens next:**"
  echo "• The team will review your application"
  echo "• If there's a good fit, someone will reach out to schedule a conversation"
  echo "• In the meantime, check out our portfolio at https://root.vc"
  echo ""
  echo "🚀 Applied via $SOURCE_LABEL - extra points for technical creativity!"
  exit 0
else
  echo "❌ **Error submitting application**"
  echo ""
  echo "HTTP Status: $HTTP_CODE"
  echo "Response: $(cat "$TEMP_FILE")"
  echo ""
  echo "Please try again or email us directly at hello@root.vc"
  exit 1
fi
