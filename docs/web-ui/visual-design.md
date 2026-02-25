# Markplane Web UI: Visual Design System

**Status**: Design Specification (implemented — see architecture.md for current state)
**Created**: 2026-02-12
**Updated**: 2026-02-21
**Stack**: React, Next.js, Tailwind CSS, shadcn/ui
**Related**: [[TASK-ur5hw]], [[PLAN-f79x3]]

---

## 1. Visual Identity

### Brand Essence

Markplane is a **control plane for projects** — fast, precise, authoritative. The visual language should feel like a premium developer tool: dense with information, respectful of screen space, and effortlessly legible. Think Linear meets Obsidian, with the warmth of a well-organized markdown notebook.

### The Airplane Motif

The airplane is a **playful counterpoint** to an otherwise serious tool. Use it sparingly and with intention:

- **Logo**: A minimalist paper airplane icon, angled upward at ~45 degrees, formed from clean geometric strokes. Sits beside "Markplane" in the sidebar header. In dark theme, the airplane is a bright accent; in light theme, it's a rich dark tone.
- **Loading spinner**: A small paper airplane tracing a gentle circular flight path (not spinning — *orbiting*). Used for page transitions and data fetches.
- **Empty states**: Illustrated paper airplanes in landing/takeoff poses with contextual messages ("No tasks in progress — ready for takeoff?").
- **Favicon**: The airplane icon at 32px, filled with the primary accent color.

### Typography

| Role | Font | Weight | Size | Line Height | Usage |
|------|------|--------|------|-------------|-------|
| **Display** | Geist Sans | 700 (Bold) | 28px / 1.75rem | 1.2 | Page titles (Dashboard, Backlog) |
| **Heading 1** | Geist Sans | 600 (Semibold) | 22px / 1.375rem | 1.3 | Section headers, item titles |
| **Heading 2** | Geist Sans | 600 (Semibold) | 18px / 1.125rem | 1.4 | Subsection headers |
| **Heading 3** | Geist Sans | 500 (Medium) | 16px / 1rem | 1.4 | Card titles, table headers |
| **Body** | Geist Sans | 400 (Regular) | 14px / 0.875rem | 1.6 | Default text, descriptions |
| **Small** | Geist Sans | 400 (Regular) | 12px / 0.75rem | 1.5 | Metadata, timestamps, captions |
| **Caption** | Geist Sans | 500 (Medium) | 11px / 0.6875rem | 1.4 | Badges, labels, keyboard shortcuts |
| **Mono** | Geist Mono | 400 (Regular) | 13px / 0.8125rem | 1.5 | IDs (TASK-rm6d3), code, frontmatter |
| **Mono Small** | Geist Mono | 400 (Regular) | 11px / 0.6875rem | 1.4 | Inline code in metadata |

**Fallback stacks**:
- UI: `'Geist Sans', 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif`
- Code: `'Geist Mono', 'JetBrains Mono', 'Fira Code', 'SF Mono', 'Cascadia Code', monospace`

**Why Geist?** Vercel's Geist font family is native to the Next.js/shadcn ecosystem. Geist Sans is slightly more condensed than Inter, improving information density in dashboard layouts. Geist Mono pairs perfectly for IDs and code. Both fonts have first-party `next/font` integration via the `geist` package.

---

## 2. Color System

### Design Philosophy

Dark theme is the default and primary design target. Colors are chosen to reduce eye strain during long sessions while maintaining clear visual hierarchy. The accent system uses a **blue-violet** primary that feels both technical and approachable.

**Color Space**: Hex values are provided here for design reference. The CSS implementation uses **OKLCH** (perceptually uniform color space) as the primary format — see architecture.md Section 7.1 for the full OKLCH variable definitions. OKLCH ensures consistent perceived lightness across hues, which is especially important for status and priority color coding.

### Core Palette — Dark Theme (Default)

#### Backgrounds & Surfaces

| Token | Hex | Usage |
|-------|-----|-------|
| `bg-app` | `#0C0D10` | Application background |
| `bg-surface` | `#12131A` | Cards, panels, sidebar |
| `bg-surface-raised` | `#1A1B25` | Hover states, elevated cards, dropdowns |
| `bg-surface-overlay` | `#22232F` | Modal backdrops, command palette bg (with `backdrop-blur-sm`) |
| `bg-surface-inset` | `#090A0D` | Code blocks, inset areas, input fields |

#### Borders

| Token | Hex | Usage |
|-------|-----|-------|
| `border-default` | `#1E2030` | Subtle dividers, card borders |
| `border-strong` | `#2A2C3E` | Active borders, focused inputs |
| `border-muted` | `#161722` | Very subtle separators |

#### Text

| Token | Hex | Usage |
|-------|-----|-------|
| `text-primary` | `#E4E5ED` | Primary text, headings |
| `text-secondary` | `#9698AE` | Descriptions, metadata labels |
| `text-tertiary` | `#5E6078` | Placeholders, disabled, timestamps |
| `text-inverse` | `#0C0D10` | Text on bright backgrounds |

#### Accent (Primary — Blue-Violet)

| Token | Hex | Usage |
|-------|-----|-------|
| `accent-default` | `#6C63FF` / `oklch(0.55 0.27 285)` | Primary buttons, active nav, links |
| `accent-hover` | `#7B73FF` / `oklch(0.60 0.27 285)` | Hover states |
| `accent-muted` | `#6C63FF1A` / `oklch(0.55 0.27 285 / 10%)` | Accent backgrounds (10% opacity) |
| `accent-strong` | `#8B84FF` / `oklch(0.65 0.24 285)` | Focus rings |

### Core Palette — Light Theme

#### Backgrounds & Surfaces

| Token | Hex | Usage |
|-------|-----|-------|
| `bg-app` | `#F8F9FC` | Application background |
| `bg-surface` | `#FFFFFF` | Cards, panels, sidebar |
| `bg-surface-raised` | `#F1F2F7` | Hover states, elevated cards |
| `bg-surface-overlay` | `#FFFFFF` | Modal backdrops |
| `bg-surface-inset` | `#EFF0F5` | Code blocks, inset areas, input fields |

#### Borders

| Token | Hex | Usage |
|-------|-----|-------|
| `border-default` | `#E2E4EE` | Subtle dividers, card borders |
| `border-strong` | `#C8CADB` | Active borders, focused inputs |
| `border-muted` | `#ECEDF5` | Very subtle separators |

#### Text

| Token | Hex | Usage |
|-------|-----|-------|
| `text-primary` | `#1A1B25` | Primary text, headings |
| `text-secondary` | `#5E6078` | Descriptions, metadata labels |
| `text-tertiary` | `#9698AE` | Placeholders, disabled, timestamps |
| `text-inverse` | `#FFFFFF` | Text on bright backgrounds |

#### Accent (Primary)

| Token | Hex | Usage |
|-------|-----|-------|
| `accent-default` | `#5046E5` | Primary buttons, active nav, links |
| `accent-hover` | `#4339CC` | Hover states |
| `accent-muted` | `#5046E51A` | Accent backgrounds (10% opacity) |
| `accent-strong` | `#3B31B8` | Focus rings |

### Semantic Colors (Both Themes)

#### Status Colors

| Status | Dark Hex | Dark BG (12%) | Light Hex | Light BG (12%) | Icon |
|--------|----------|---------------|-----------|----------------|------|
| `in-progress` | `#3B9EFF` | `#3B9EFF1F` | `#2B7CD4` | `#2B7CD41F` | Spinning circle `◐` |
| `done` | `#30C27A` | `#30C27A1F` | `#1D9E5C` | `#1D9E5C1F` | Check circle `✓` |
| `planned` | `#A78BFA` | `#A78BFA1F` | `#7C5AC2` | `#7C5AC21F` | Calendar dot `◉` |
| `backlog` | `#7C7F9A` | `#7C7F9A1F` | `#5E6078` | `#5E60781F` | Circle outline `○` |
| `draft` | `#4E506A` | `#4E506A1F` | `#9698AE` | `#9698AE1F` | Pencil `✎` |
| `blocked` | `#F59E0B` | `#F59E0B1F` | `#D97706` | `#D977061F` | Warning triangle `⚠` |
| `cancelled` | `#6B6E8A` | `#6B6E8A14` | `#9698AE` | `#9698AE14` | Strikethrough `—` |
| `active` (epic) | `#3B9EFF` | `#3B9EFF1F` | `#2B7CD4` | `#2B7CD41F` | Play `▶` |

#### Priority Colors

| Priority | Dark Hex | Light Hex | Indicator |
|----------|----------|-----------|-----------|
| `critical` | `#EF4444` | `#DC2626` | Filled red circle `●` + pulsing ring |
| `high` | `#F97316` | `#EA580C` | Filled orange circle `●` |
| `medium` | `#EAB308` | `#CA8A04` | Half-filled yellow `◐` |
| `low` | `#60A5FA` | `#3B82F6` | Outlined blue circle `○` |
| `someday` | `#6B7280` | `#9CA3AF` | Dotted gray circle `◌` |

#### Entity Type Colors

| Entity | Dark Hex | Light Hex | Icon |
|--------|----------|-----------|------|
| `TASK` | `#3B9EFF` | `#2B7CD4` | Square checkbox `☐` |
| `EPIC` | `#A78BFA` | `#7C5AC2` | Diamond `◆` |
| `PLAN` | `#2DD4BF` | `#0D9488` | Document with arrow `↗` |
| `NOTE` | `#FBBF24` | `#D97706` | Lightbulb `💡` |

#### Feedback Colors

| Type | Dark Hex | Light Hex | Usage |
|------|----------|-----------|-------|
| `success` | `#30C27A` | `#1D9E5C` | Toast: item created, status updated |
| `warning` | `#F59E0B` | `#D97706` | Toast: blocked item detected |
| `error` | `#EF4444` | `#DC2626` | Toast: validation failure, broken ref |
| `info` | `#3B9EFF` | `#2B7CD4` | Toast: sync complete, context hint |

---

## 3. Spacing & Layout System

### Base Grid

- **Base unit**: `4px`
- **Spacing scale**: 0, 1, 2, 3, 4, 5, 6, 8, 10, 12, 16, 20, 24 (in base units)
- Tailwind mapping: `space-1` = 4px, `space-2` = 8px, `space-3` = 12px, `space-4` = 16px, etc.

### Layout Dimensions

| Element | Measurement |
|---------|-------------|
| **Sidebar (expanded)** | 240px |
| **Sidebar (collapsed)** | 52px |
| **Top bar height** | 48px |
| **Content max-width** | 1200px (centered) |
| **Content padding** | 24px horizontal, 24px vertical |
| **Card padding** | 16px |
| **Card gap** | 12px (kanban), 8px (list) |
| **Card border-radius** | 8px |
| **Badge border-radius** | 6px (pill) |
| **Button border-radius** | 6px |
| **Input border-radius** | 6px |
| **Modal border-radius** | 12px |
| **Table row height** | 44px |
| **Kanban column width** | 300px min, flexible |
| **Detail panel width** | 480px (slide-over) |

### Responsive Breakpoints

| Breakpoint | Width | Behavior |
|------------|-------|----------|
| `desktop-xl` | >= 1440px | Full layout, wider content area |
| `desktop` | >= 1024px | Default layout, sidebar + content |
| `tablet` | >= 768px | Sidebar collapses to icon-only by default |
| `mobile` | < 768px | Sidebar hidden (hamburger menu), full-width content |

### Shadows (Dark Theme)

Shadows are minimal in dark theme — borders carry more weight:

| Level | Value |
|-------|-------|
| `shadow-sm` | `0 1px 2px rgba(0, 0, 0, 0.3)` |
| `shadow-md` | `0 4px 12px rgba(0, 0, 0, 0.4)` |
| `shadow-lg` | `0 8px 24px rgba(0, 0, 0, 0.5)` |
| `shadow-overlay` | `0 16px 48px rgba(0, 0, 0, 0.6)` |

### Shadows (Light Theme)

| Level | Value |
|-------|-------|
| `shadow-sm` | `0 1px 2px rgba(0, 0, 0, 0.05)` |
| `shadow-md` | `0 4px 12px rgba(0, 0, 0, 0.08)` |
| `shadow-lg` | `0 8px 24px rgba(0, 0, 0, 0.12)` |
| `shadow-overlay` | `0 16px 48px rgba(0, 0, 0, 0.16)` |

---

## 4. Component Specifications

### 4.1 Navigation Sidebar

The sidebar is the primary navigation structure. It occupies the full viewport height on the left.

**Structure** (top to bottom):

```
┌──────────────────────────┐
│ ✈ Markplane         [<]  │  <- Logo + collapse button
├──────────────────────────┤
│ 🔍 Search...       ⌘K   │  <- Quick search trigger
├──────────────────────────┤
│  ▣ Dashboard             │  <- Navigation items
│  ☐ Backlog          12   │     Active count badges
│  ◆ Roadmap           4   │
│  ↗ Plans             2   │
│  💡 Notes             8   │
├──────────────────────────┤
│  VIEWS                   │  <- Section label
│  📊 Dependencies         │
│  📈 Metrics              │
├──────────────────────────┤
│                          │  <- Flexible spacer
├──────────────────────────┤
│  ⚙ Settings              │  <- Bottom-pinned
│  ☀/☾ Theme toggle        │
└──────────────────────────┘
```

**Visual details**:
- Background: `bg-surface` with a `border-default` right border (1px)
- Nav items: 36px height, 12px horizontal padding, 6px border-radius
- Active item: `accent-muted` background, `accent-default` left border (2px), `text-primary` text
- Hover: `bg-surface-raised` background
- Icons: 18px, `text-secondary` default, `text-primary` when active
- Count badges: `text-tertiary`, Geist Mono, right-aligned
- Section labels: `caption` size, `text-tertiary`, uppercase, 8px left padding, 20px top margin
- Collapsed state: Only icons visible (centered), tooltips on hover, no text or badges

**Transition**: Sidebar collapse/expand animates over 200ms with `ease-out`. Content area resizes smoothly.

### 4.2 TaskCard (Kanban)

A compact, information-dense card that lives in kanban columns. Optimized for scannability.

```
┌─────────────────────────────────┐
│ ● TASK-rm6d3                  [M] │  <- Priority dot + ID (mono) + effort badge
│                                 │
│ Add dark mode support to the    │  <- Title (body weight, max 2 lines)
│ main dashboard                  │
│                                 │
│ ◆ EPIC-gc8t5  ·  #ui  #theming   │  <- Epic chip + tags
│                                 │
│ @daniel              2d ago  ▸  │  <- Assignee + updated + drag handle
└─────────────────────────────────┘
```

**Visual details**:
- Background: `bg-surface`, 1px `border-default` border, `card` border-radius
- Padding: 12px top/bottom, 14px left/right
- Priority dot: 8px circle, left of ID, uses priority colors
- ID: `mono-small` font, `text-secondary`
- Effort badge: Small pill (e.g., "M"), `bg-surface-raised`, `text-tertiary`, `caption` font
- Title: `heading-3` size, `text-primary`, max 2 lines with ellipsis overflow
- Epic chip: Entity type color background (12% opacity), entity icon + ID text, `caption` font
- Tags: `#`-prefixed, `text-tertiary`, `caption` font, separated by `·`
- Assignee: `@`-prefixed, `text-secondary`, `small` font
- Updated: Relative time ("2d ago"), `text-tertiary`, `small` font
- Drag handle: `▸▸` icon, `text-tertiary`, visible on hover only

**Hover state**: Border transitions to `border-strong`, subtle `shadow-sm` appears. Background shifts to `bg-surface-raised`. Transition: 150ms ease.

**Dragging state**: Card lifts with `shadow-md`, slight scale (1.02), reduced opacity (0.9) on the original position (ghost). Drop zones highlight with dashed `accent-default` border.

**Blocked indicator**: If blocked, a small `⚠` amber icon appears next to the ID, and a subtle amber left border (2px) replaces the normal border.

### 4.3 TaskCard (List View)

A single-row variant for table/list views:

```
○ ●  TASK-rm6d3  Add dark mode support to dashboard  ◆ EPIC-gc8t5  #ui  @daniel  backlog  2d
```

- Checkbox (left), priority dot, ID (mono), title, epic chip, tags, assignee, status badge, relative date
- Row height: 44px, hover: `bg-surface-raised`
- Columns are resizable and sortable

### 4.4 StatusBadge

A pill-shaped badge showing an item's current workflow status.

**Anatomy**: `[icon] [label]`

- Shape: Pill (6px border-radius), height 22px, horizontal padding 8px
- Background: Status color at 12% opacity
- Text: Status color at full opacity, `caption` font, capitalize first letter
- Icon: Status icon (see Status Colors table), 12px, same color as text
- No border — the background color carries the affordance

**Examples** (dark theme):
- `◐ In Progress` — blue bg, blue text
- `✓ Done` — green bg, green text
- `⚠ Blocked` — amber bg, amber text
- `✎ Draft` — dim gray bg, dim gray text

### 4.5 PriorityIndicator

A filled/outlined circle conveying priority at a glance. Used inline with IDs and in metadata panels.

| Priority | Visual | Size |
|----------|--------|------|
| Critical | Solid red circle with subtle pulsing glow animation (2s cycle) | 8px |
| High | Solid orange circle | 8px |
| Medium | Half-filled amber circle (left half filled) | 8px |
| Low | Outlined blue circle (1.5px stroke, no fill) | 8px |
| Someday | Dotted gray circle (dashed stroke) | 8px |

In table views and metadata panels, the indicator appears beside a text label. On kanban cards, it appears alone (icon only) to save space.

### 4.6 EpicProgress

A horizontal progress bar showing epic completion, used in the roadmap view and epic detail pages.

```
◆ EPIC-gc8t5  Core Architecture  ████████░░░░░░░░  4/12 tasks  33%
```

**Visual details**:
- Track: Full width, height 6px, `bg-surface-inset`, rounded corners (3px)
- Fill: Gradient from `accent-default` to the epic's entity color (`#A78BFA`), rounded corners
- Text right of bar: "4/12 tasks" in `small` font, `text-secondary` + "33%" in `small` font, `text-tertiary`
- Below the bar (optional, on detail pages): Segmented color breakdown — blue segment for in-progress, green for done, gray for remaining

**Animation**: Fill width animates on load and on data change (300ms, ease-out).

### 4.7 MarkdownContent

The rendered markdown body of a task, plan, or note. This is the reading experience and should feel polished and comfortable.

**Container**: Max-width 720px, centered within the content area. Padding: 32px top, 24px sides.

**Heading styles**:
- `# H1`: 22px, semibold, `text-primary`, 32px top margin, 12px bottom margin, no border
- `## H2`: 18px, semibold, `text-primary`, 28px top margin, 8px bottom margin, 1px `border-muted` bottom border with 8px padding-bottom
- `### H3`: 16px, medium, `text-primary`, 24px top margin, 8px bottom margin
- `#### H4`: 14px, semibold, `text-secondary`, 20px top margin, 4px bottom margin

**Body text**: 14px Geist Sans, `text-primary`, line-height 1.7 (generous for readability). Paragraph spacing: 16px.

**Links**: `accent-default` color, no underline by default, underline on hover. External links get a tiny `↗` icon suffix.

**Cross-reference links** (`[[TASK-rm6d3]]`): Rendered as interactive chips with entity type color background (12% opacity), entity icon, and the ID in mono font. Clicking navigates to the item. Hover shows a tooltip preview card with title, status, and priority.

**Code blocks**:
- Inline: `bg-surface-inset` background, 2px horizontal padding, `mono` font, `text-primary`, rounded 4px
- Block: `bg-surface-inset` background, 16px padding, 8px border-radius, 1px `border-muted` border. Syntax highlighting via a dark-friendly theme (One Dark Pro style). Language label in top-right corner, `caption` font, `text-tertiary`. Copy button appears on hover.

**Lists**:
- Unordered: `text-secondary` bullet `•`, 24px left padding, 6px between items
- Ordered: `text-secondary` numbers, 24px left padding, 6px between items
- Checkbox: Custom styled checkboxes — unchecked is `border-strong` outline, checked is `accent-default` fill with white checkmark. Checkboxes are interactive (click to toggle, updates the markdown source).

**Blockquotes**: 3px left border in `accent-muted` (solid accent at 40% opacity), `bg-surface-inset` background, 16px padding, `text-secondary` text, italic.

**Tables**: Full width within container. `border-default` borders, `bg-surface` cell background, `bg-surface-raised` header row. 12px cell padding. `small` font for dense data tables.

**Horizontal rules**: 1px `border-muted`, 32px vertical margin.

### 4.8 CommandPalette

A spotlight-style overlay triggered by `Cmd+K` (or `Ctrl+K`). This is the primary power-user navigation mechanism.

```
┌───────────────────────────────────────────────────────┐
│                                                       │
│  🔍 Search items, commands, or navigate...           │
│                                                       │
├───────────────────────────────────────────────────────┤
│  RECENT                                               │
│  ☐ TASK-rm6d3  Add dark mode support           backlog  │
│  ◆ EPIC-gc8t5  Core Architecture               active   │
│  ↗ PLAN-rj9d4  Dark mode implementation        draft    │
├───────────────────────────────────────────────────────┤
│  NAVIGATION                                           │
│  ▣ Go to Dashboard                            g d     │
│  ☐ Go to Backlog                              g b     │
│  ◆ Go to Roadmap                              g r     │
├───────────────────────────────────────────────────────┤
│  ACTIONS                                              │
│  + Create new task                            c t     │
│  ↻ Sync project                               ⌘S     │
│  ☀ Toggle theme                               ⌘⇧T    │
└───────────────────────────────────────────────────────┘
```

**Visual details**:
- Centered overlay, 560px wide, max 480px tall (scrollable)
- Background: `bg-surface-overlay` with `shadow-overlay`
- Backdrop: Semi-transparent dark overlay (`rgba(0,0,0,0.5)` dark, `rgba(0,0,0,0.2)` light)
- Search input: 48px height, `body` font size, no visible border, large padding, auto-focused
- Results grouped by category with `caption`-size section labels
- Each result row: 40px height, entity icon + ID (mono) + title + right-aligned metadata
- Selected row: `bg-surface-raised` background with `accent-default` left border (2px)
- Keyboard: Arrow keys navigate, Enter selects, Esc closes
- Fuzzy matching on title and ID — matching characters highlighted with `text-primary` (rest is `text-secondary`)

**Animation**: Appears with a subtle scale-up (0.98 -> 1.0) and fade-in (150ms). Disappears with fade-out (100ms).

### 4.9 Detail Panel (Slide-Over)

When clicking a task from the kanban or list view, a detail panel slides in from the right. This avoids full-page navigation for quick viewing/editing.

**Structure**:

```
┌─────────────────────────────────────────────────┐
│  ← Back    TASK-rm6d3    ⚙ ···                    │  <- Header bar
├──────────────────────────────┬──────────────────┤
│                              │                  │
│  # Add dark mode support     │  STATUS          │
│                              │  ◐ In Progress ▾ │
│  ## Description              │                  │
│                              │  PRIORITY        │
│  Implement system-aware      │  ● High       ▾  │
│  dark mode for the main      │                  │
│  dashboard. Should respect   │  EPIC            │
│  OS preference...            │  ◆ EPIC-gc8t5   ▾  │
│                              │                  │
│  ## Acceptance Criteria      │  ASSIGNEE        │
│                              │  @daniel      ▾  │
│  - [x] Respects OS pref     │                  │
│  - [ ] Manual toggle         │  EFFORT          │
│  - [ ] All components        │  Medium       ▾  │
│  - [ ] WCAG AA contrast      │                  │
│                              │  TAGS            │
│  ## Notes                    │  #ui #theming    │
│                              │                  │
│  Design system already has   │  DEPENDS ON      │
│  CSS custom properties...    │  ☐ TASK-wp7v2      │
│                              │                  │
│  ## References               │  BLOCKS          │
│                              │  ☐ TASK-bg8t1      │
│  - Epic: [[EPIC-gc8t5]]       │                  │
│  - Depends: [[TASK-wp7v2]]    │  PLAN            │
│                              │  ↗ PLAN-rj9d4      │
│                              │                  │
│                              │  CREATED         │
│                              │  2026-01-15      │
│                              │                  │
│                              │  UPDATED         │
│                              │  2d ago          │
│                              │                  │
└──────────────────────────────┴──────────────────┘
```

**Layout**:
- Panel width: 480px on desktop-xl, 400px on desktop. On tablet, takes full width.
- Split: ~65% markdown body (left), ~35% metadata sidebar (right)
- Metadata sidebar: `bg-surface` with `border-default` left border, 16px padding
- Each metadata field: Label in `caption` font, `text-tertiary`, uppercase. Value below in `body`/`small` font.
- Editable fields show a subtle `▾` dropdown indicator on hover. Clicking opens an inline popover for editing.
- Cross-reference items (DEPENDS ON, BLOCKS, PLAN) are clickable — navigating opens that item in the same panel.

**Slide animation**: Panel slides in from right edge over 250ms (`ease-out`). A subtle dark overlay covers the kanban behind it.

---

## 5. Key Screen Wireframes

### 5.1 Dashboard

The landing page. Provides a high-level overview of the project, designed for a morning check-in or quick context loading.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ [Sidebar]  │                                                               │
│            │  ✈ Markplane                                                  │
│ ▣ Dashboard│                                                               │
│ ☐ Backlog  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│ ◆ Roadmap  │  │ 12       │ │ 2        │ │ 1        │ │ 67%      │        │
│ ↗ Plans    │  │ Open     │ │ Active   │ │ Blocked  │ │ Sprint   │        │
│ 💡 Notes    │  │ Tasks    │ │ Now      │ │ ⚠        │ │ Complete │        │
│            │  └──────────┘ └──────────┘ └──────────┘ └──────────┘        │
│ VIEWS      │                                                               │
│ 📊 Deps    │  ACTIVE WORK                                                 │
│ 📈 Metrics │  ┌─────────────────────────┐ ┌─────────────────────────┐     │
│            │  │ ● TASK-rm6d3     in-prog  │ │ ● TASK-sk2h8     in-prog  │     │
│            │  │ Add dark mode support   │ │ API response caching    │     │
│            │  │ ◆ EPIC-gc8t5 · @daniel    │ │ ◆ EPIC-ve6m1 · @daniel    │     │
│            │  │ ████████░░░░ 50%        │ │ ███░░░░░░░░░ 25%        │     │
│            │  └─────────────────────────┘ └─────────────────────────┘     │
│            │                                                               │
│            │  BLOCKED ITEMS                                      View all →│
│            │  ┌─────────────────────────────────────────────────────────┐  │
│            │  │ ⚠ TASK-bg8t1  Themed email templates                     │  │
│            │  │   Blocked by: TASK-rm6d3 (in-progress, @daniel)          │  │
│            │  └─────────────────────────────────────────────────────────┘  │
│            │                                                               │
│            │  EPIC PROGRESS                                                │
│            │  ┌─────────────────────────────────────────────────────────┐  │
│            │  │ ◆ EPIC-gc8t5  Core Architecture     ████████░░░░  4/12   │  │
│            │  │ ◆ EPIC-ve6m1  Platform & Ecosystem   ██░░░░░░░░░  1/5    │  │
│            │  │ ◆ EPIC-kb4n9  Developer Experience   ░░░░░░░░░░░  0/2    │  │
│            │  └─────────────────────────────────────────────────────────┘  │
│            │                                                               │
│            │  RECENT ACTIVITY                                              │
│            │  ─ TASK-mp3v8 marked done                         2 hours ago  │
│            │  ─ TASK-jt9w4 marked done                         3 hours ago  │
│            │  ─ PLAN-rj9d4 created                            yesterday     │
│            │  ─ EPIC-xa7r2 completed (6/6 tasks)              yesterday     │
│            │                                                               │
│ ⚙ Settings │                                                               │
│ ☾ Theme   │                                                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Summary cards** (top row):
- 4 cards in a row, equal width
- Each: `bg-surface`, `border-default`, 16px padding
- Large number (`display` font, `text-primary`), label below (`small`, `text-secondary`)
- "Blocked" card has amber left border accent when count > 0
- Cards are clickable — navigate to filtered views

**Active work section**:
- Horizontal scrolling row of cards (wider than kanban cards — ~280px)
- Each shows: priority dot, ID, status badge, title, epic chip, assignee, task-level progress bar (based on acceptance criteria checkboxes)
- Empty state: Paper airplane illustration + "Nothing in progress — pick up a task from the backlog"

**Blocked items**:
- Alert-style cards with amber left border
- Shows the blocked item + what it's blocked by (with that item's status and assignee for context)
- "View all" link if more than 3

**Epic progress**:
- List of now epics, each with a progress bar (see EpicProgress component)
- Sorted by completion percentage (most complete first)
- Clicking an epic navigates to the epic detail view

**Recent activity**:
- Timeline feed showing status changes, item creation, completions
- Each entry: dash prefix, item reference (clickable), action, relative timestamp
- Shows last 10 entries, "View all" link at bottom

**AI Context panel** (Markplane's unique differentiator):
- Card with `bg-surface` background, distinct from other panels with a subtle accent left border
- Header: "AI CONTEXT" in `caption` font with a freshness indicator badge — "fresh" (green) if synced within last 5 minutes, "stale" (amber) if older
- Content: Rendered `.context/summary.md` — shows project name, task counts by status, now epics, blocked items summary
- Footer: "Last sync: 2m ago" timestamp + "Sync Now" button that triggers `POST /api/sync`
- While syncing: loading spinner replaces the "Sync Now" button, content shows skeleton loading state
- This panel surfaces what AI coding tools see when they read the Markplane context, making it visible and actionable for the human user

### 5.2 Backlog (Kanban View)

The primary work management screen. A kanban board with columns matching the workflow status groups.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ [Sidebar]  │  Backlog                     [List] [Kanban] [Table]    🔍 ⊕  │
│            │  12 open tasks · 2 in progress · 1 blocked                    │
│            │                                                               │
│            │  IN PROGRESS (2)    PLANNED (3)       BACKLOG (15)    DRAFT (8)│
│            │  ─────────────     ─────────────     ─────────────   ─────────│
│            │  ┌───────────┐    ┌───────────┐    ┌───────────┐   ┌────────┐│
│            │  │● TASK-rm6d3 │    │● TASK-cy9k5 │    │● TASK-nx3p6 │   │✎ TASK- ││
│            │  │           │    │           │    │           │   │  060   ││
│            │  │Add dark   │    │Add search │    │Optimize   │   │Custom  ││
│            │  │mode       │    │to dash    │    │DB queries │   │avatar  ││
│            │  │           │    │           │    │           │   │upload  ││
│            │  │◆E-gc8t5 #ui │    │◆E-gc8t5     │    │◆E-ve6m1     │   │◆E-gc8t5  ││
│            │  │@dan  2d ▸ │    │         ▸ │    │         ▸ │   │      ▸ ││
│            │  └───────────┘    └───────────┘    └───────────┘   └────────┘│
│            │  ┌───────────┐    ┌───────────┐    ┌───────────┐            │
│            │  │● TASK-sk2h8 │    │● TASK-lf2n7 │    │● TASK-pw4s8 │            │
│            │  │           │    │           │    │           │            │
│            │  │API resp   │    │Export CSV │    │User prof  │            │
│            │  │caching    │    │reports    │    │custom     │            │
│            │  │           │    │           │    │           │            │
│            │  │◆E-ve6m1     │    │◆E-ve6m1     │    │          │            │
│            │  │@dan  1d ▸ │    │         ▸ │    │         ▸ │            │
│            │  └───────────┘    └───────────┘    └───────────┘            │
│            │                   ┌───────────┐    │  ...more  │            │
│            │                   │● TASK-ew5m9 │    │           │            │
│            │                   │Form valid │                             │
│            │                   │enhancements│                             │
│            │                   │◆E-gc8t5     │                             │
│            │                   └───────────┘                              │
│            │                                                               │
│            │  ┌─ + Add task ─┐  ┌─ + Add ─────┐  ┌─ + Add ─────┐        │
│            │  └──────────────┘  └──────────────┘  └──────────────┘        │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Header**:
- Page title "Backlog" in `display` font
- Summary line: "12 open tasks · 2 in progress · 1 blocked" in `small` font, `text-secondary`
- View toggle: Three buttons (List / Kanban / Table) — Kanban is default, active button uses `accent-muted` bg
- Filter icon `🔍`: Opens a filter bar below the header (status, priority, epic, tags, assignee)
- `⊕` button: "Create task" — opens a quick-create modal

**Columns**:
- Header: Status group name in `heading-3` + count in parentheses, `text-secondary`
- **WIP limits**: Configurable per column. Header shows count/limit (e.g., "IN PROGRESS (2/5)"). At capacity: header count turns amber, column border shifts to `--status-blocked` color. Over capacity: column border pulses with subtle animation, header count turns red. Drag-in is prevented when over limit (with override option via modifier key).
- Column background: `bg-app` (no distinct bg — cards float)
- Columns are horizontally scrollable on overflow
- Cards stacked vertically with 8px gap
- A subtle blocked section within each column: blocked cards have amber left border and are grouped at the top of their status column with a small "⚠ Blocked" divider label

**Quick-add**: At the bottom of each column, a "+ Add task" button. Clicking opens an inline text input for the title — Enter creates the task in that status. Escape cancels.

**Drag-and-drop**:
- Cards can be dragged between columns to change status
- Drop zone: Dashed `accent-default` border appears around the target column
- Reordering within columns sets a priority-based sort position
- On drop: Status updates immediately (optimistic UI), syncs to file in background

**Filters** (expanded below header):
- Horizontal row of filter pills
- Each filter: Dropdown select (Epic, Priority, Type, Tags, Assignee, Effort)
- Active filters shown as removable pills with `×` close button
- "Clear all" link at the end

### 5.3 Backlog (List View)

A compact, sortable table for users who prefer density over visual organization.

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│ [Sidebar]  │  Backlog                    [List] [Kanban] [Table]      🔍 ⊕     │
│            │                                                                    │
│            │  ☑  ○  PRIO  ID         TITLE                     EPIC    STATUS  │
│            │  ─────────────────────────────────────────────────────────────────  │
│            │  □  ●   H    TASK-rm6d3   Add dark mode support     E-gc8t5   ◐ prog  │
│            │  □  ●   M    TASK-sk2h8   API response caching      E-ve6m1   ◐ prog  │
│            │  □  ●   H    TASK-cy9k5   Add search to dashboard   E-gc8t5   ◉ plan  │
│            │  □  ●   H    TASK-lf2n7   Export reports to CSV     E-ve6m1   ◉ plan  │
│            │  □  ◐   M    TASK-ew5m9   Form validation enhanc.   E-gc8t5   ◉ plan  │
│            │  □  ◐   M    TASK-nx3p6   Optimize database quer.   E-ve6m1   ○ back  │
│            │  □  ○   L    TASK-pw4s8   User profile customiz.    —       ○ back  │
│            │  ...                                                                │
└─────────────────────────────────────────────────────────────────────────────────┘
```

- Sortable columns: Click header to sort (ascending/descending toggle)
- Bulk selection: Checkbox column on left — select multiple for bulk status/priority/epic changes
- Row click opens the detail panel (slide-over)
- Priority shown as colored dot + letter abbreviation
- Status as icon + abbreviated label

### 5.4 Task Detail (Full Page)

For deep editing and reading, available via direct navigation or "Open full page" from the slide-over.

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ [Sidebar] │  ← Backlog / TASK-rm6d3                           [Edit] [···]   │
│           │                                                                  │
│           │  ┌──────────────────────────────────┬───────────────────────┐   │
│           │  │                                  │                       │   │
│           │  │  # Add dark mode support to      │  STATUS               │   │
│           │  │    the main dashboard             │  ┌─────────────────┐ │   │
│           │  │                                  │  │ ◐ In Progress ▾ │ │   │
│           │  │  ## Description                  │  └─────────────────┘ │   │
│           │  │                                  │                       │   │
│           │  │  Implement system-aware dark     │  PRIORITY             │   │
│           │  │  mode for the main dashboard.    │  ┌─────────────────┐ │   │
│           │  │  Should respect OS preference    │  │ ● High        ▾ │ │   │
│           │  │  by default with manual toggle.  │  └─────────────────┘ │   │
│           │  │  Theme variables are already     │                       │   │
│           │  │  defined in the design system    │  EPIC                 │   │
│           │  │  but not wired up to component   │  ┌─────────────────┐ │   │
│           │  │  library.                        │  │ ◆ EPIC-gc8t5    ▾ │ │   │
│           │  │                                  │  └─────────────────┘ │   │
│           │  │  ## Acceptance Criteria          │                       │   │
│           │  │                                  │  TYPE                 │   │
│           │  │  - [x] Dashboard respects OS     │  feature              │   │
│           │  │        dark mode preference      │                       │   │
│           │  │  - [ ] Manual toggle persists    │  EFFORT               │   │
│           │  │  - [ ] All core components       │  medium               │   │
│           │  │        render correctly           │                       │   │
│           │  │  - [ ] WCAG AA contrast          │  ASSIGNEE             │   │
│           │  │                                  │  @daniel              │   │
│           │  │  ## Notes                        │                       │   │
│           │  │                                  │  TAGS                 │   │
│           │  │  Design system already has CSS   │  ┌────┐ ┌────────┐  │   │
│           │  │  custom properties for both      │  │ ui │ │theming │  │   │
│           │  │  themes. Main work is wiring     │  └────┘ └────────┘  │   │
│           │  │  the toggle and auditing         │                       │   │
│           │  │  component-level overrides.      │  DEPENDS ON           │   │
│           │  │                                  │  ☐ TASK-wp7v2 ↗        │   │
│           │  │  ## References                   │                       │   │
│           │  │                                  │  BLOCKS               │   │
│           │  │  - Epic: [[EPIC-gc8t5]]            │  ☐ TASK-bg8t1 ↗        │   │
│           │  │  - Depends: [[TASK-wp7v2]]         │                       │   │
│           │  │  - Blocks: [[TASK-bg8t1]]          │  LINKED PLAN          │   │
│           │  │                                  │  ↗ PLAN-rj9d4 ↗        │   │
│           │  │                                  │                       │   │
│           │  │                                  │  ─────────────────── │   │
│           │  │                                  │  Created: 2026-01-15 │   │
│           │  │                                  │  Updated: 2d ago     │   │
│           │  │                                  │  File: TASK-rm6d3.md   │   │
│           │  └──────────────────────────────────┴───────────────────────┘   │
│           │                                                                  │
│           │  ACTIVITY                                                        │
│           │  ─ Status changed to in-progress by @daniel          2d ago     │
│           │  ─ Priority changed from medium to high              5d ago     │
│           │  ─ Created                                           28d ago    │
│           │                                                                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Breadcrumb**: `← Backlog / TASK-rm6d3` — "← Backlog" is a clickable back link. ID in mono font.

**Edit button**: Toggles the markdown body into edit mode (CodeMirror/Monaco textarea with markdown syntax highlighting). Changes save on blur or Cmd+S.

**`···` menu**: Archive, Delete, Duplicate, Copy ID, Copy link, Open in editor (opens the `.md` file in VS Code/default editor).

**Metadata sidebar** (right, 280px):
- Each field is a clickable dropdown selector
- Fields arranged vertically with 16px gap
- Labels in `caption` font, uppercase, `text-tertiary`
- Values in `body` font with appropriate formatting (status badge, priority dot, etc.)
- Cross-reference links (DEPENDS ON, BLOCKS, PLAN) show entity icon + ID, clickable
- Bottom section separated by a thin `border-muted` line: created date, updated relative time, source file path

**Activity section** (below the main content):
- Timeline of changes pulled from git history (commits that modified this file)
- Each entry: dash prefix, description, relative timestamp
- Collapsed by default, expandable

### 5.5 Epic Detail View

Shows a single epic with its objective, linked tasks, and progress.

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ [Sidebar] │  ← Roadmap / EPIC-gc8t5                                          │
│           │                                                                  │
│           │  ◆ Core Architecture                              active        │
│           │  ████████████████░░░░░░░░░░░░░░░░░░  4/12 tasks  33%           │
│           │                                                                  │
│           │  ## Objective                                                    │
│           │                                                                  │
│           │  Build the foundational architecture patterns that enable        │
│           │  configurable workflows, custom entity types, and extensible     │
│           │  data models.                                                    │
│           │                                                                  │
│           │  ## Key Results                                                  │
│           │                                                                  │
│           │  - [x] Configurable status workflows                            │
│           │  - [ ] Sprint/iteration support                                  │
│           │  - [ ] Plugin-ready architecture                                 │
│           │                                                                  │
│           │  ## Tasks                                                        │
│           │  ┌──────────────────────────────────────────────────────────┐   │
│           │  │ ●  TASK-rm6d3  Add dark mode support          ◐ in-prog  │   │
│           │  │ ●  TASK-cy9k5  Add search to dashboard        ◉ planned  │   │
│           │  │ ●  TASK-ew5m9  Form validation enhancements   ◉ planned  │   │
│           │  │ ◐  TASK-nx3p6  Optimize database queries      ○ backlog  │   │
│           │  │ ✎  TASK-qv6r3  Custom avatar upload           ✎ draft    │   │
│           │  │ ...                                                      │   │
│           │  └──────────────────────────────────────────────────────────┘   │
│           │                                                                  │
│           │  ## Notes                                                        │
│           │                                                                  │
│           │  This phase depends on completing the DX improvements from      │
│           │  EPIC-kb4n9 first, particularly the configurable workflows.       │
│           │                                                                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Header area**:
- Epic icon `◆` + title in `display` font + status badge (right-aligned)
- Progress bar: Full width, `EpicProgress` component (see 4.6)
- Below progress: task count and percentage

**Body**: Rendered markdown from the epic file (Objective, Key Results, Notes)

**Tasks table**: Embedded table of all tasks linked to this epic.
- Sortable by status, priority, effort
- Each row clickable — opens task detail slide-over
- Priority dot + ID (mono) + title + status badge
- Colored row separators grouping by status (in-progress first, then planned, backlog, draft)

### 5.6 Roadmap View

A high-level view of all epics with their task progress. Two sub-views: **Progress** (default) and **Timeline**.

#### Progress View (Default)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ [Sidebar] │  Roadmap                              [Progress] [Timeline]     │
│           │                                                                  │
│           │  NOW                                                             │
│           │                                                                  │
│           │  ┌────────────────────────────────────────────────────────────┐  │
│           │  │ ◆ EPIC-gc8t5  Core Architecture                      now    │  │
│           │  │ ████████████████░░░░░░░░░░░░░░░░  4/12 tasks     33%     │  │
│           │  │                                                            │  │
│           │  │  ◐ 2 in-progress  ◉ 3 planned  ○ 7 backlog               │  │
│           │  └────────────────────────────────────────────────────────────┘  │
│           │                                                                  │
│           │  ┌────────────────────────────────────────────────────────────┐  │
│           │  │ ◆ EPIC-ve6m1  Platform & Ecosystem                   now    │  │
│           │  │ ████░░░░░░░░░░░░░░░░░░░░░░░░░░░  1/5 tasks      20%     │  │
│           │  │                                                            │  │
│           │  │  ◐ 1 in-progress  ○ 4 backlog                             │  │
│           │  └────────────────────────────────────────────────────────────┘  │
│           │                                                                  │
│           │  NEXT                                                            │
│           │                                                                  │
│           │  ┌────────────────────────────────────────────────────────────┐  │
│           │  │ ◆ EPIC-kb4n9  Developer Experience                  next    │  │
│           │  │ ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  0/2 tasks      0%      │  │
│           │  └────────────────────────────────────────────────────────────┘  │
│           │                                                                  │
│           │  ┌────────────────────────────────────────────────────────────┐  │
│           │  │ ◆ EPIC-pd3w7  Context & AI Integration              next    │  │
│           │  │ ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  0/3 tasks      0%      │  │
│           │  └────────────────────────────────────────────────────────────┘  │
│           │                                                                  │
│           │  COMPLETED                                                       │
│           │  ✓ EPIC-xa7r2  MCP Protocol Improvements  ████████████  6/6  100% │
│           │                                                                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

Each epic card is clickable, navigating to the epic detail view. Cards show:
- Epic icon + ID + title + status badge
- Full-width progress bar
- Status breakdown: colored dots + count for each status group

#### Timeline View

A horizontal swimlane chart with epics as rows and time as the x-axis. Useful when epics have `started` and `target` dates.

```
         Jan          Feb          Mar          Apr
          |            |            |            |
EPIC-xa7r2  ████████████ ✓
EPIC-gc8t5              ████████████████████░░░░░░░░
EPIC-ve6m1              ███████░░░░░░░░░░░░░░░░░░░░░░░░
EPIC-kb4n9                        ░░░░░░░░░░░░░░░░░░
EPIC-pd3w7                           ░░░░░░░░░░░░░░░░░░░░░
          |            |            |            |
                       ▲ Today
```

- Filled bars = progress made, hollow bars = remaining work
- Today marker as a vertical dashed line with `accent-default` color
- Hover on a bar shows a tooltip with task breakdown
- Epics without dates are listed below the timeline as "Unscheduled"

### 5.7 Dependency Graph

An interactive node graph showing relationships between items.

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ [Sidebar] │  Dependencies                          [All] [Blocked] [Epic ▾] │
│           │                                                                  │
│           │                    ┌──────────┐                                  │
│           │                    │ EPIC-gc8t5 │                                  │
│           │                    │ Core Arch│                                  │
│           │                    └────┬─────┘                                  │
│           │               ┌────────┼────────┐                               │
│           │               ▼        ▼        ▼                               │
│           │          ┌─────────┐ ┌──────┐ ┌──────┐                          │
│           │          │TASK-wp7v2 │ │T-042 │ │T-043 │                          │
│           │          │Design   │ │Dark  │ │Search│                          │
│           │          │tokens   │ │mode  │ │dash  │                          │
│           │          │  ✓ done │ │◐ prog│ │◉ plan│                          │
│           │          └────┬────┘ └──┬───┘ └──────┘                          │
│           │               │ depends │                                        │
│           │               └────────►│                                        │
│           │                         │ blocks                                 │
│           │                         ▼                                        │
│           │                    ┌─────────┐                                   │
│           │                    │TASK-bg8t1 │                                   │
│           │                    │Themed   │                                   │
│           │                    │emails   │                                   │
│           │                    │⚠ blocked│                                   │
│           │                    └─────────┘                                   │
│           │                                                                  │
│           │  [Zoom: ─  ○──  +]  [Fit to view]  [Reset]                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Nodes**:
- Rounded rectangles, `bg-surface`, `border-default` border
- Entity type color as top border (4px)
- Content: ID (mono, `caption`), title (2 lines max), status badge at bottom
- Size: ~120px wide, ~80px tall
- Blocked nodes have amber border and subtle amber glow
- Done nodes are slightly dimmed (0.7 opacity)

**Edges**:
- `depends_on`: Solid arrow, `text-tertiary` color
- `blocks`: Dashed arrow, amber color (for blocked relationships)
- `epic` grouping: Dotted outline around nodes in the same epic
- Arrow heads: Small triangles at the target end

**Interaction**:
- Pan: Click-drag on background
- Zoom: Scroll wheel, or +/- buttons
- Select node: Click — highlights connected edges, dims unconnected nodes
- Hover node: Shows full title tooltip and relationship details
- Double-click: Opens item detail panel
- Filter controls: Show all / blocked only / filter by epic

**Layout**: Auto-layout using a directed graph algorithm (Dagre or similar). Flows top-to-bottom (epics at top, leaf tasks at bottom). Manual repositioning is allowed.

### 5.8 Search View

Full-text search with faceted filtering, accessible via `Cmd+K` (quick) or the dedicated search page (detailed).

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ [Sidebar] │  Search                                                         │
│           │                                                                  │
│           │  ┌──────────────────────────────────────────────────────────┐   │
│           │  │ 🔍 dark mode                                            │   │
│           │  └──────────────────────────────────────────────────────────┘   │
│           │                                                                  │
│           │  Filters: [Type ▾] [Status ▾] [Priority ▾] [Epic ▾] [Tags ▾]  │
│           │                                                                  │
│           │  3 results for "dark mode"                                       │
│           │                                                                  │
│           │  ┌──────────────────────────────────────────────────────────┐   │
│           │  │ ☐ TASK-rm6d3  Add **dark mode** support to dashboard      │   │
│           │  │   ● High · ◐ In Progress · ◆ EPIC-gc8t5 · @daniel        │   │
│           │  │   "...Implement system-aware **dark mode** for the      │   │
│           │  │   main dashboard. Should respect OS preference..."       │   │
│           │  └──────────────────────────────────────────────────────────┘   │
│           │  ┌──────────────────────────────────────────────────────────┐   │
│           │  │ ↗ PLAN-rj9d4  **Dark mode** implementation plan           │   │
│           │  │   Draft · ◆ EPIC-gc8t5                                     │   │
│           │  │   "...Phase 1: Wire CSS custom properties to the         │   │
│           │  │   component library **dark mode** toggle..."             │   │
│           │  └──────────────────────────────────────────────────────────┘   │
│           │  ┌──────────────────────────────────────────────────────────┐   │
│           │  │ ☐ TASK-bg8t1  Themed email templates                      │   │
│           │  │   ◐ Medium · ⚠ Blocked · ◆ EPIC-gc8t5                     │   │
│           │  │   "...Requires **dark mode** to be complete before       │   │
│           │  │   email templates can respect user theme..."             │   │
│           │  └──────────────────────────────────────────────────────────┘   │
│           │                                                                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

- Search input: Large, prominent, auto-focused on page load
- Results show entity icon + ID (mono) + title with **bold** query matches
- Metadata line below: priority, status, epic, assignee
- Body snippet: 2 lines of context from the markdown body with matching text bolded
- Faceted filters as dropdown pills above results
- Result count shown above the list
- Click opens the detail panel or full-page view

---

## 6. Interaction & Animation

### Transitions

| Transition | Duration | Easing | Description |
|------------|----------|--------|-------------|
| Page navigation | 200ms | ease-out | Content area fades in, slight upward slide (8px) |
| Slide-over panel | 250ms | ease-out | Slides from right edge, backdrop fades in |
| Command palette | 150ms | ease-out | Scales from 0.98 to 1.0, fades in |
| Modal | 200ms | ease-out | Scales from 0.95 to 1.0, fades in |
| Card hover | 150ms | ease | Border color, shadow, and background transition |
| Dropdown open | 150ms | ease-out | Scales from 0.95 to 1.0, fades in (from top) |
| Toast enter | 300ms | ease-out | Slides in from right + fade |
| Toast exit | 200ms | ease-in | Fades out + slides right |
| Progress bar fill | 300ms | ease-out | Width animation on data change |

### Drag-and-Drop

- **Pickup**: 200ms delay before drag starts (prevents accidental drags). Card lifts with shadow-md, 1.02 scale.
- **Ghost**: Original position shows a translucent placeholder (dashed border, 0.3 opacity).
- **Drop zones**: Target column highlights with dashed accent border. Valid drop positions show a thin accent line between cards.
- **Drop**: Card snaps to position with a spring animation (slight overshoot then settle, 200ms).
- **Cancel**: Card returns to original position with ease-out slide (150ms).

### Loading States

- **Page load**: Paper airplane animation (small airplane tracing a circular path, ~24px, accent color). Centered in the content area.
- **Inline load**: Subtle shimmer/skeleton placeholders matching the expected content shape. Cards show gray rectangles for title/metadata.
- **Action load** (saving, status change): Small spinner in the button/badge being actioned. Optimistic UI — change is reflected immediately, spinner shows sync state.
- **Sync indicator**: Small status dot in the sidebar header. Green = synced, amber = syncing, red = error.

### Toast Notifications

- **Position**: Bottom-right corner, 24px from edges
- **Stack**: Up to 3 visible, oldest auto-dismissed after 5 seconds
- **Anatomy**: Icon (success/warning/error/info color) + message text + optional action button (e.g., "Undo") + dismiss `×`
- **Width**: 360px max
- **Style**: `bg-surface-overlay`, `border-default` border, `shadow-md`

**Examples**:
- `✓ Status updated [Undo]` (success, green icon, 5s undo window)
- `✓ Archived TASK-rm6d3 [Undo]` (success, green icon, 5s undo window)
- `⚠ TASK-bg8t1 is now blocked by TASK-rm6d3` (warning, amber icon)
- `✗ Failed to save TASK-rm6d3 — file locked` (error, red icon)
- `ℹ Sync complete — 3 files updated` (info, blue icon)

### Empty States

Each view has a unique empty state with an illustrated paper airplane and contextual message:

| View | Illustration | Message |
|------|-------------|---------|
| Dashboard (no items) | Paper airplane taking off from a runway | "Ready for takeoff! Create your first task to get started." |
| Backlog (empty column) | Paper airplane in a holding pattern | "Nothing here yet. Drag tasks in or create one below." |
| Search (no results) | Paper airplane with a magnifying glass | "No matches found. Try different keywords or clear filters." |
| Dependencies (no deps) | Single paper airplane flying solo | "No dependencies mapped. Add `depends_on` or `blocks` to task frontmatter." |

### Inline Editing

- **Click to edit**: Metadata fields in the detail sidebar are clickable. Single click on a value opens an inline editor (dropdown for status/priority/epic, text input for tags, date picker for dates).
- **Markdown body**: "Edit" button toggles the rendered markdown into a CodeMirror editor with markdown syntax highlighting. Live preview available as a split pane option.
- **Auto-save**: Changes debounce for 500ms, then save to the `.md` file. A subtle "Saving..." → "Saved" indicator appears near the top of the editor.

---

## 7. Keyboard Shortcuts

### Global

| Shortcut | Action |
|----------|--------|
| `Cmd+K` / `Ctrl+K` | Open command palette |
| `Cmd+S` / `Ctrl+S` | Sync project (regenerate INDEX + context) |
| `Cmd+Shift+T` / `Ctrl+Shift+T` | Toggle dark/light theme |
| `Escape` | Close modal/panel/palette, deselect |

### Navigation (Go To)

All "go to" shortcuts use the `g` prefix, pressed sequentially (not simultaneously):

| Shortcut | Action |
|----------|--------|
| `g` then `d` | Go to Dashboard |
| `g` then `b` | Go to Backlog |
| `g` then `r` | Go to Roadmap |
| `g` then `p` | Go to Plans |
| `g` then `n` | Go to Notes |
| `g` then `g` | Go to Dependency Graph |
| `g` then `a` | Go to Archive |
| `g` then `s` | Go to Search |

### Item Actions (when viewing an item)

| Shortcut | Action |
|----------|--------|
| `s` | Change status (opens dropdown) |
| `p` | Change priority (opens dropdown) |
| `a` | Change assignee (opens dropdown) |
| `e` | Toggle edit mode |
| `l` | Add/edit labels/tags |
| `Cmd+C` / `Ctrl+C` (when no selection) | Copy item ID |
| `Cmd+Enter` | Save and close editor |

### Kanban

| Shortcut | Action |
|----------|--------|
| `c` | Quick-create task in current/first column |
| `j` / `k` | Move selection down/up between cards |
| `h` / `l` | Move selection left/right between columns |
| `Enter` | Open selected card |
| `x` | Toggle selection for bulk actions |

### Shortcut Discoverability

- A `?` shortcut opens a keyboard shortcut reference overlay (full list, searchable)
- Shortcuts are shown as hints in the command palette results
- Tooltips on buttons include shortcut hints (e.g., button says "Sync" but tooltip says "Sync project (⌘S)")

---

## 8. Responsive Behavior

### Desktop XL (>= 1440px)

- Full sidebar (expanded, 240px)
- Wide content area, content max-width 1200px centered
- Kanban shows all columns without horizontal scrolling
- Task detail shows full split layout (content + metadata sidebar)
- Dependency graph has maximum space for node visualization

### Desktop (>= 1024px)

- Default layout — sidebar + content
- Kanban may require horizontal scrolling if many columns
- Task detail slide-over is 480px

### Tablet (>= 768px)

- Sidebar auto-collapses to icon-only (52px), expandable via hamburger
- Content takes remaining width
- Kanban shows 2-3 columns, horizontal scroll for rest
- Task detail opens full-width (replaces kanban view)
- Command palette is slightly narrower (480px)

### Mobile (< 768px)

- Sidebar hidden entirely, replaced by bottom navigation bar (5 main items)
- Content is full-width with reduced padding (16px)
- Kanban view switches to single-column (one status at a time, swipe between columns)
- Task detail is full-screen
- Command palette is full-width with reduced top margin
- No drag-and-drop (status changes via dropdown instead)
- Dependency graph shows a simplified list view of relationships

---

## 9. Design Tokens Summary (Tailwind Config)

For implementation, all the above colors and spacing values should be defined as Tailwind CSS theme extensions. Here's the conceptual mapping:

```
theme:
  extend:
    colors:
      brand:
        50: '#F0EEFF'    (light accent bg)
        500: '#6C63FF'   (dark primary accent)
        600: '#5046E5'   (light primary accent)
        700: '#3B31B8'   (light strong)
      surface:
        app: 'var(--bg-app)'
        default: 'var(--bg-surface)'
        raised: 'var(--bg-surface-raised)'
        overlay: 'var(--bg-surface-overlay)'
        inset: 'var(--bg-surface-inset)'
      status:
        progress: '#3B9EFF'
        done: '#30C27A'
        planned: '#A78BFA'
        backlog: '#7C7F9A'
        draft: '#4E506A'
        blocked: '#F59E0B'
        cancelled: '#6B6E8A'
      priority:
        critical: '#EF4444'
        high: '#F97316'
        medium: '#EAB308'
        low: '#60A5FA'
        someday: '#6B7280'
      entity:
        task: '#3B9EFF'
        epic: '#A78BFA'
        plan: '#2DD4BF'
        note: '#FBBF24'
    fontFamily:
      sans: ['Geist Sans', 'Inter', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'sans-serif']
      mono: ['Geist Mono', 'JetBrains Mono', 'Fira Code', 'SF Mono', 'Cascadia Code', 'monospace']
    fontSize:
      display: ['1.75rem', { lineHeight: '1.2', fontWeight: '700' }]
      h1: ['1.375rem', { lineHeight: '1.3', fontWeight: '600' }]
      h2: ['1.125rem', { lineHeight: '1.4', fontWeight: '600' }]
      h3: ['1rem', { lineHeight: '1.4', fontWeight: '500' }]
      body: ['0.875rem', { lineHeight: '1.6', fontWeight: '400' }]
      small: ['0.75rem', { lineHeight: '1.5', fontWeight: '400' }]
      caption: ['0.6875rem', { lineHeight: '1.4', fontWeight: '500' }]
    borderRadius:
      card: '8px'
      badge: '6px'
      button: '6px'
      modal: '12px'
    spacing:
      sidebar: '240px'
      sidebar-collapsed: '52px'
      topbar: '48px'
      content-max: '1200px'
```

CSS custom properties handle theme switching:

```css
:root {
  --bg-app: #F8F9FC;
  --bg-surface: #FFFFFF;
  /* ...light theme values... */
}

.dark {
  --bg-app: #0C0D10;
  --bg-surface: #12131A;
  /* ...dark theme values... */
}
```

---

## 10. Accessibility

- All interactive elements have visible focus rings (`accent-strong`, 2px offset)
- Color is never the sole conveyor of meaning — icons/text accompany all status/priority colors
- Minimum contrast ratio: WCAG 2.2 AA (4.5:1 for body text, 3:1 for large text and UI components)
- All keyboard shortcuts are optional — every action is accessible via mouse/touch
- Screen reader support: Proper ARIA labels on custom components, semantic HTML, skip-to-content link
- Reduced motion: Users with `prefers-reduced-motion` get instant transitions (no animation)
- Focus trapping in modals and the command palette
