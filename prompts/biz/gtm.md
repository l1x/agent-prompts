```yaml
kind: prompt
name: gtm
description: "Go-to-market strategy, positioning, and messaging architecture"
inputs:
  - name: mission
    required: true
  - name: context
    required: false
    default: ""
outputs:
  - name: strategy
    format: markdown
```

## Mission

{{mission}}

## Instructions

### Phase 1: Strategic Foundation

Before producing any content, clarify these questions (ask if not provided in mission/context):

1. What customer truth have you discovered that competitors miss?
2. What's the 3-word category you own? (e.g., "Sales Intelligence Platform")
3. Who's your ideal customer profile (title, company size, pain level)?
4. What's your unfair advantage (tech, data, team, insight)?
5. What's the one metric that proves your value?

### Phase 2: Messaging Architecture

Produce a messaging hierarchy:

- **Primary value prop** — hero headline (5-7 words)
- **Supporting benefits** — 3 max (any more dilutes focus)
- **Proof points** — metrics, logos, testimonials
- **Differentiators** — vs. status quo, vs. competitors
- **Call-to-action** — the clear next step

### Phase 3: Content by Section

For each deliverable section, provide:

- **Strategic rationale** — why this section matters
- **Content hierarchy** — headline → sub-head → body
- **Conversion goal** — what visitors should do/feel
- **Proof points** — what builds credibility

## Frameworks

Apply these as appropriate:

- **Jobs-to-be-Done** for value propositions
- **Challenger Sale** methodology for positioning
- **StoryBrand** framework for narrative flow
- **AIDA** (Attention, Interest, Desire, Action) for conversion paths

## Output

```markdown
## GTM Strategy

### Positioning

[Category, target customer, key differentiator]

### Messaging Architecture

[Headline, sub-headline, supporting benefits, proof points]

### Competitive Positioning

[How you win vs. status quo and named competitors]

### Content Recommendations

[Section-by-section content with strategic rationale]

### Investor Signals

[Elements that signal traction, clarity, inevitability, ambition]
```

## Constraints

- Specific beats superlative — "5x ROI" beats "amazing results"
- Quantify everything — "3x faster" not "faster"
- Clarity beats cleverness — if you confuse, you lose
- One target, one message — speaking to everyone speaks to no one
- Customer first — establish "you" (customer pain) before "we" (solution)
- No vague positioning: avoid "best," "leading," "innovative" without proof
- No feature lists without outcomes

## Context from prior steps

{{context}}
