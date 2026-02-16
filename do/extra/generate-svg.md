# Diagram Design System

Scandinavian-inspired technical diagrams: minimal, premium, functional.

> **Last Updated:** 2026-02-05

## Core Principles

1. **No solid color fills** - Always use border + light pastel fill
2. **Different things = different colors** - Semantic categories must be visually distinct
3. **White space is a feature** - Let elements breathe
4. **Restraint over decoration** - Every element must earn its place
5. **Borders define, fills emphasize** - Use fills sparingly for semantic meaning

## Color Palette

### Backgrounds & Surfaces

| Role            | Hex       | Usage             |
| --------------- | --------- | ----------------- |
| Canvas          | `#fafafa` | Page background   |
| Surface         | `#ffffff` | Cards, containers |
| Border light    | `#e2e8f0` | Default borders   |
| Border emphasis | `#cbd5e1` | Active elements   |

### Text

| Role      | Hex       | Usage                    |
| --------- | --------- | ------------------------ |
| Primary   | `#1e293b` | Headings, labels         |
| Secondary | `#64748b` | Descriptions, metadata   |
| Muted     | `#94a3b8` | Disabled, scaled-to-zero |

## Semantic Categories

All categories use border + light fill, never solid fills.

## Container Headers

Use thin accent line (4px) at top, not solid header bar.

```svg
<rect fill="#fff" stroke="#e2e8f0" stroke-width="1"/>
<rect y="0" height="4" fill="#ea580c"/>  <!-- accent line -->
<text fill="#ea580c">Container Name</text>
```

## Typography

| Element  | Font        | Size | Weight  | Color          |
| -------- | ----------- | ---- | ------- | -------------- |
| Title    | `monospace` | 18px | bold    | `#1e293b`      |
| Subtitle | `system-ui` | 14px | regular | `#64748b`      |
| Label    | `monospace` | 12px | bold    | category color |
| Body     | `system-ui` | 10px | regular | `#64748b`      |

## Layout Rules

- Minimum 15px padding inside containers
- Minimum 20px between major sections
- Thin accent lines (4px) instead of solid header bars
- Use `stroke-width="1"` for most elements, `"1.5"` for emphasis
- Corner radius: `rx="4"` for cards, `rx="8"` for containers

## Arrow Markers

```svg
<defs>
  <!-- Default gray arrow -->
  <marker id="arrowhead" markerWidth="8" markerHeight="6" refX="7" refY="3" orient="auto">
    <polygon points="0 0, 8 3, 0 6" fill="#94a3b8"/>
  </marker>

  <!-- Green arrow for abstract interface connections -->
  <marker id="arrowhead-green" markerWidth="8" markerHeight="6" refX="7" refY="3" orient="auto">
    <polygon points="0 0, 8 3, 0 6" fill="#059669"/>
  </marker>

  <!-- Orange arrow for infrastructure connections -->
  <marker id="arrowhead-orange" markerWidth="8" markerHeight="6" refX="7" refY="3" orient="auto">
    <polygon points="0 0, 8 3, 0 6" fill="#ea580c"/>
  </marker>
</defs>
```
