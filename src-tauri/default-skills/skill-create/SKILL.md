---
name: skill-create
description: Use when creating a new skill, writing a SKILL.md, improving an existing skill, or teaching an LLM a new workflow. Triggers on: create skill, new skill, write skill, skill-create, build skill, improve skill, optimize skill.
---

# Skill Creator

You create skills that teach LLMs new capabilities. A skill is a structured
prompt (SKILL.md) that transforms an LLM into a specialist for a specific task.

<critical_constraints>
- Description = trigger conditions ONLY, max 250 characters. Beyond 250 chars,
  the text is truncated — the LLM never sees the rest.
- If the description explains HOW instead of WHEN, the LLM thinks it already
  understands and skips the body. This is the #1 cause of skill failure.
- All instructions in the body = direct imperative 2nd person ("You write...",
  "You check..."). NEVER use descriptive infinitive ("Writing...", "Checking...").
  Infinitives are perceived as suggestions and the agent skips steps.
- SKILL.md must stay under 500 lines. Move excess content to references/.
- Scripts must be stdlib-only — zero pip/npm install.
</critical_constraints>

---

## Phase 1 — Understand the Intent

Before writing anything, you clarify:

| Question | Why it matters |
|----------|---------------|
| What should the skill enable? | Defines the scope — one skill = one capability |
| When should it trigger? | Becomes the description — trigger phrases, not explanations |
| What's the expected output format? | Shapes the instructions (JSON, markdown, file, etc.) |
| What data does it receive? | Determines what goes in system prompt vs user prompt |

You extract answers from the current conversation first. If the user already
demonstrated a workflow ("turn this into a skill"), you capture the tools used,
the sequence of steps, corrections made, and input/output formats observed.

You ask about edge cases, input formats, success criteria, and dependencies.
You do not start writing until intent is clear.

---

## Phase 2 — Anatomy of a Skill

```
skill-name/
├── SKILL.md              ← Required. Under 500 lines.
│   ├── YAML frontmatter  ← name, description (required)
│   └── Markdown body     ← Instructions, examples, workflows
├── scripts/              ← Optional. Deterministic code (0 tokens if not run)
├── references/           ← Optional. Detailed docs (0 tokens if not read)
└── assets/               ← Optional. Templates, icons, fonts
```

### Three-level loading

1. **Metadata** (name + description) — always in context (~100 words)
2. **SKILL.md body** — loaded when skill triggers (< 500 lines)
3. **Bundled resources** — loaded on demand (unlimited size)

You keep SKILL.md lean. If a section serves less than 30% of use cases,
you move it to `references/` with a clear pointer from the body.

---

## Phase 3 — The 10 Rules of Skill Writing

### Rule 1 — Description = triggers only

The description determines whether the LLM invokes the skill. Nothing else matters
if the description fails.

<important>
HARD LIMIT: 250 characters. Beyond 250 chars, the description is truncated in
session context — the LLM never sees the end.

If the description contains instructions ("Converts videos using H264 compression"),
the LLM thinks it already understands and skips reading the body entirely.
</important>

Measured data on 200+ prompts:

| Description type | Trigger rate |
|-----------------|-------------|
| Vague ("Helps with files") | ~20% |
| Optimized (what + when) | ~50% |
| With trigger examples | 72-90% |
| Explanatory summary | ~0% — LLM skips the body |
| Over 250 chars | Truncated — tail triggers lost |

```yaml
# BAD — explanatory, LLM skips the body
description: Converts videos with ffmpeg using H264 compression and audio extraction

# GOOD — triggers only
description: Use when converting, compressing, or extracting audio from video.
  Triggers on: ffmpeg, video, compress, transcode, extract audio.
```

### Rule 2 — Imperative 2nd person, never infinitive

You write every instruction as a direct order: "You write...", "You check...",
"Send the file...". Never use descriptive infinitives ("Writing...", "Checking...").

LLM agents treat infinitives as suggestions and skip steps. Direct imperative
leaves no room for interpretation — the agent executes.

```markdown
# BAD — infinitive, perceived as "here's what we could do"
## Step 2 — Generate the summary
Building a prompt with the changelog. Sending the result via API.

# GOOD — direct imperative, the agent executes
## Step 2 — Generate the summary
You build a prompt with the changelog. You send the result via API.
```

You also cover the "nothing to do" case explicitly. If there's no work needed,
you state what the agent should say — otherwise it stops silent.

### Rule 3 — XML tags for structure

You use XML tags to create clear boundaries between different types of information.
Tags help the LLM understand what each section contains and refer back to it.

```xml
<task_context>
You are an AI assistant helping a claims adjuster review car accident reports.
</task_context>

<form_data>
{{FORM_IMAGE}}
</form_data>

<instructions>
1. You examine the form first, then the sketch.
2. You list every checked box with its meaning.
3. You determine fault based on the evidence.
</instructions>

<output_format>
You wrap your final verdict in <final_verdict> tags.
</output_format>
```

XML tags are preferred over markdown headers for data boundaries because:
- They explicitly name what's inside (not just a heading)
- They're token-efficient
- They allow nesting and clear open/close semantics

### Rule 4 — Enforcement tags

For rules that must NEVER be violated, you wrap them in enforcement tags.
Regular instructions can be deprioritized by the LLM. Enforcement tags signal
non-negotiable constraints.

```xml
<critical_constraints>
- You NEVER expose API keys in logs or error messages.
- You ALWAYS validate input before processing. Fail closed.
- You compare secrets in constant-time only — never use ==.
</critical_constraints>

<important>
If the form is unreadable, you state this immediately and do not proceed
with the analysis. Do not guess.
</important>
```

Use `<critical_constraints>` for security rules and hard invariants.
Use `<important>` for behavioral rules that prevent common mistakes.
Use sparingly — if everything is critical, nothing is.

### Rule 5 — Order of analysis matters

When the skill processes multiple data sources, you specify the exact order
in which the LLM must analyze them. The order impacts reasoning quality.

```markdown
## Analysis order
1. You read the form FIRST — extract all factual data points.
2. You summarize what you know so far in <accident_summary> tags.
3. THEN you examine the sketch, keeping your summary in mind.
4. You correlate sketch details with form data before concluding.
```

The principle: give the LLM structured context before ambiguous content.
A form with checkboxes provides facts. A hand-drawn sketch is ambiguous.
Reading facts first anchors interpretation of the ambiguous content.

### Rule 6 — Pre-fill to force output format

You can force the LLM to start its response in a specific format by
pre-filling the beginning of the assistant response.

```markdown
## Output format
You begin your response with the opening tag. No preamble.

Pre-fill example (API): set the assistant message to start with `<final_verdict>`
Pre-fill example (skill): "You wrap your final output in <result> tags.
Do not write anything before the opening <result> tag."
```

This eliminates preamble ("Sure! Here's my analysis...") and makes
the output directly parseable by the calling application.

### Rule 7 — Quick Start is mandatory

The first 3-5 commands or examples the LLM sees must cover 80% of use cases.
If the LLM has to read 200 lines before knowing what to do, the skill fails.

```markdown
## Quick Start
1. You read the input file and validate the format.
2. You run the analysis: `python ${CLAUDE_SKILL_DIR}/scripts/analyze.py data.json`
3. You format the output as markdown with a summary section.
```

### Rule 8 — Few-shot examples

Examples act as concrete templates. It's often more efficient to show
the LLM one or two desired outputs than to describe every nuance in text.

```xml
<example>
Input: User uploads a CSV with columns: date, revenue, costs
Output:
<analysis>
## Financial Summary
- Period: 2024-01 to 2024-12
- Total revenue: $1.2M
- Total costs: $890K
- Net margin: 25.8%
</analysis>
</example>
```

Guidelines for examples:
- **Relevance**: examples must match the actual use case
- **Diversity**: cover edge cases and gray areas, not just the happy path
- **Quantity**: 3-5 examples minimum for tasks with consistent formatting
- You wrap each example in `<example>` tags for clear boundaries

### Rule 9 — Static data in system prompt

Data that never changes between invocations goes in the system prompt:
form structures, database schemas, API specs, company policies.

Data that changes per request goes in the user prompt:
the actual form content, user query, uploaded files.

This separation enables prompt caching — the static part is cached,
reducing cost and latency on repeated calls.

```markdown
## System prompt (static, cacheable)
<form_structure>
The Swedish car accident form has 17 rows and 2 columns (Vehicle A / Vehicle B).
Row 1: "Was parked/stationary" ...
</form_structure>

## User prompt (dynamic, per-request)
<form_image>
{{UPLOADED_FORM}}
</form_image>
```

### Rule 10 — Final reminder and guidelines

At the end of the skill, you repeat the most critical rules. The LLM pays
more attention to instructions at the beginning and end of a prompt —
the middle gets less weight (primacy/recency effect).

```markdown
<important>
Before delivering your response:
- You base every claim on visible evidence. No assumptions.
- If data is unclear, you state this explicitly instead of guessing.
- You wrap your final output in <result> tags.
</important>
```

---

## Phase 4 — Progressive Disclosure

You keep SKILL.md under 500 lines. Content that serves less than 30%
of use cases goes to `references/`.

| Content | Keep in SKILL.md | Move to references/ |
|---------|-----------------|-------------------|
| Quick Start | Always | |
| Core commands (top 20) | Yes | |
| Advanced commands | | api-reference.md |
| Common workflows (top 3) | Yes | |
| Specialized workflows | | workflows.md |
| Troubleshooting | | troubleshooting.md |
| Full parameter lists | | options.md |

You organize by domain when a skill supports multiple frameworks:

```
cloud-deploy/
├── SKILL.md (workflow + selection logic)
└── references/
    ├── aws.md
    ├── gcp.md
    └── azure.md
```

The LLM reads only the relevant reference file — 0 tokens for the others.

---

## Phase 5 — Anti-patterns

| Anti-pattern | Problem | Fix |
|-------------|---------|-----|
| **Explanatory description** | LLM reads description, thinks it knows, skips body | Description = triggers only ("Use when...") |
| **Infinitive instructions** | Agent treats as suggestion, skips steps | Imperative 2nd person: "You write...", "You send..." |
| **No Quick Start** | LLM reads 200 lines before acting | First 3-5 items cover 80% of cases |
| **Everything is critical** | Enforcement tags lose their power | Use `<critical_constraints>` sparingly |
| **No examples** | LLM guesses the output format | Add 3-5 few-shot examples |
| **Monolithic SKILL.md** | Over 500 lines, LLM loses focus | Split into references/ |
| **No edge case handling** | Agent stops silent when nothing to do | Cover the "nothing to do" case explicitly |
| **Kitchen sink description** | Over 250 chars, tail is truncated | Keep it to trigger conditions only |
| **Ambiguous data order** | LLM analyzes sources in wrong order | Specify exact analysis sequence |

---

## Phase 6 — STOP CHECK

You run this checklist before delivering. Every item must pass.

- [ ] Description = trigger conditions only, max 250 chars, no instructions
- [ ] Body instructions = imperative 2nd person ("You..."), never infinitive
- [ ] "Nothing to do" case handled explicitly
- [ ] Quick Start with 3-5 items covering 80% of cases
- [ ] XML tags for data boundaries (`<context>`, `<task>`, `<output>`)
- [ ] Enforcement tags used for non-negotiable rules (`<critical_constraints>`)
- [ ] Few-shot examples with `<example>` tags (3-5 minimum)
- [ ] Static data separated from dynamic data
- [ ] Analysis order specified when multiple data sources
- [ ] Final reminder repeats critical rules
- [ ] SKILL.md under 500 lines (advanced content in references/)
- [ ] Links to `references/` for detailed content (one level only)
- [ ] Zero hardcoded secrets (env vars only)
- [ ] Scripts stdlib-only (zero external dependencies)