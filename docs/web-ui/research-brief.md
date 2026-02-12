# Markplane Web UI — Design Research Brief

**Status**: Reference
**Created**: 2026-02-12
**Updated**: 2026-02-12
**Related**: [[TASK-017]], [[PLAN-001]]

> Comprehensive research into modern PM tools, developer dashboards, markdown-first UIs, and design trends (2025-2026) to inform Markplane's web UI design.

---

## 1. Modern PM Tool UI Analysis

### Linear — The Gold Standard

Linear has defined the modern PM aesthetic. Key takeaways:

**Typography**
- **Inter Display** for headings (added expression), **Inter** for body text
- Bold typography as a staple — creative typeface usage over generic sans-serifs
- Multiple size scales: title-6, text-regular, text-small, text-mini

**Color System**
- Migrated from HSL to **LCH color space** for perceptually uniform color relationships
- Theme generation reduced from 98 variables to just **3 per theme**: base color, accent color, contrast
- Contrast variable (range 30-100) enables automatic high-contrast accessibility themes
- Primary brand color: subtle desaturated blue, comfortable on both light and dark backgrounds
- 2025 trend: monochrome black/white with fewer bold accent colors

**Dark Theme Philosophy**
- Not pure black — brand color at 1-10% lightness for warmth
- Text and neutral icons lighter in dark mode for improved contrast
- Users define just background, text, and accent — system generates complementary shades for borders and elevated surfaces

**Layout & Navigation**
- **Inverted L-shape navigation**: sidebar + top bar as global chrome
- Reduced visual noise — blue accent usage deliberately limited
- One-dimensional scrolling, consistently aligned text
- Logical top-to-bottom, left-to-right progression
- Nearly **every action keyboard-accessible** — Cmd+K command palette is central

**Key Design Principles**
- Neutral, timeless appearance over trendy elements
- Density optimization — tabs, headers, filters adjusted to reduce clutter
- Cross-platform consistency (macOS, Windows, browser)
- Multiple view modes: list, board, timeline, split, fullscreen

### Plane (plane.so) — Open-Source Linear Alternative

- Clean, modern UI focused on usability and responsiveness
- **Multiple views out of the box**: Kanban, List, Gantt, Calendar, Spreadsheet
- Built-in AI assistance for workflow automation
- Docker/Kubernetes deployment with lightweight footprint
- Emphasis on workflow fluidity and developer focus

### Huly — All-in-One Platform

- Unified dashboard consolidating projects, notifications, and recent activities
- Visual Gantt charts, Kanban boards, and list views
- Combined project planning, note-taking, task management, communication
- Sleek modern design prioritizing simplicity without sacrificing depth
- Responsive across desktop, tablet, and mobile

### Height — AI-Native PM (Sunset Sept 2025)

- Beautiful, intuitive, fast interface
- AI studied workspace patterns to proactively handle backlog upkeep, spec updates, bug triage
- Multiple visualization modes: Spreadsheet, Kanban, Gantt, Calendar
- Noteworthy for its AI-first approach to reducing administrative overhead

### Shortcut — Developer-Focused

- Story/Objective/Epic hierarchy for goal-based progression
- Drag-and-drop Kanban boards for intuitive prioritization
- Reporting tools with cumulative flow diagrams and burndown charts
- Deep GitHub/GitLab/Slack integrations
- Intuitive interface appreciated as a lighter Jira alternative

**Key Patterns Across PM Tools:**
| Pattern | Adoption |
|---------|----------|
| Dark-first theme | Linear, Plane, Huly |
| Cmd+K command palette | Linear, Height |
| Multiple views (kanban/list/timeline) | All tools |
| Keyboard-first interaction | Linear, Height |
| AI-assisted workflows | Height, Plane, Huly |
| Minimal chrome, high content density | Linear, Shortcut |

---

## 2. Developer Dashboard UI Analysis

### Vercel Dashboard

**Design Philosophy**
- Pure blacks `oklch(0 0 0)`, pure whites `oklch(1 0 0)` — minimal palette carried by typography and spacing
- "The design equivalent of writing clean code — every element justified, nothing wasted"
- **Geist** design system for consistency

**Developer-Centric UX**
- Prioritizes production deployment status, git connection, deployment links
- Command Menu for keyboard navigation (developer-oriented workflows)
- Scope selector for team switching
- Card view / list view toggle
- Performance focus: decreased First Meaningful Paint by 1.2+ seconds

**Geist Design System**
- **Geist Sans**: legibility-focused, Swiss typography principles
- **Geist Mono**: monospaced for code, terminals, diagrams
- OKLCH color tokens for perceptual uniformity
- Built-in light/dark theme switcher

### Railway

- Beautiful, intuitive dashboard with real-time log and service status views
- Real-time collaboration — teammates see updates without refreshing
- Centralized: building, domain provisioning, SSL, backups, logging all in one view
- Modern UI with usage-based metrics displayed cleanly

### Render

- Logs, metrics, monitoring dashboards out of the box
- OpenTelemetry integration for metrics streaming
- Comprehensive visibility: application performance, resource usage, system health

### GitHub Projects v2

- Board layout with customizable columns (Status field or any single-select field)
- Multi-item drag between columns with automatic value adjustment
- Column limits (WIP limits) for kanban discipline
- Multiple layout options: table (high-density), board (kanban), timeline (roadmap)
- Spreadsheet-style alternative to card-based approach

**Key Patterns Across Developer Dashboards:**
| Pattern | Tools |
|---------|-------|
| Pure black/white dark theme | Vercel |
| Real-time status updates | Railway, Render |
| Keyboard-first navigation | Vercel |
| High data density | GitHub Projects, Vercel |
| OKLCH/LCH color space | Vercel, Linear |
| Monospaced font for code | Vercel (Geist Mono) |

---

## 3. Markdown-First Tool UI Analysis

### Obsidian

- **Theme ecosystem**: extensive community themes (Encore, Minimal, etc.) — all CSS-customizable
- **Canvas**: whiteboard-like space for spatial note organization
- **Multimedia embedding**: PDFs, videos, audio, entire web pages
- **Wiki-links**: `[[note]]` syntax with bidirectional linking
- **Community plugins**: massive ecosystem for workflow customization
- Local-first, files stored as plain `.md` — aligns directly with Markplane philosophy

### Notion

- Structured, visual approach — blocks-based content editing
- Seamless blend of rich text editing and database views
- Inline databases with multiple views (table, board, timeline, gallery, calendar)
- Clean typography with generous whitespace
- Cover images and emoji icons for visual warmth

### Logseq

- Outliner-based markdown editing
- Bidirectional linking with graph visualization
- Local-first, privacy-focused
- Block-level referencing and embedding

**Key Patterns for Markdown Rendering:**
- Clean typography with generous line-height (1.6-1.8)
- Syntax highlighting for code blocks with copy button
- Frontmatter rendered as clean metadata badges/chips
- Wiki-links rendered as interactive, styled inline links
- Headers with anchor links for navigation
- Task checkboxes rendered as interactive elements
- Tables rendered with clean borders and alternating row colors

---

## 4. Design Trends 2025-2026

### Command Palettes (Cmd+K)

- Central interaction pattern for power users
- Not just search — **for doing things** (navigate, create, update, filter)
- **Best practices:**
  - Hint about palette in UI — don't rely solely on keyboard shortcut
  - Context-aware: global commands + contextual actions based on current view
  - Display associated keyboard shortcuts alongside each command
  - Fast, accurate fuzzy search is critical
  - Visual feedback on command execution
  - Include everything from menus and context menus
- Adopted by: Linear, GitHub, Netlify, Vercel, VS Code, Obsidian

### Glassmorphism

- Frosted glass-like translucent surfaces
- Used by: Windows 11, iOS Control Center, macOS Big Sur, Spotify
- Combines visual detail with readability
- **For Markplane**: Use sparingly — modal overlays, command palette backdrop, sidebar on hover
- Risk: overuse makes everything look the same

### Micro-Animations

- Context-specific: slight bounce, smooth fade, quick shift on updates
- Motion design getting "quieter but smarter" in 2026
- Key areas: status transitions, drag-and-drop feedback, page transitions, loading states
- Framer Motion is the go-to React library

### AI-Integrated Interfaces

- AI as quiet assistant, not novelty
- In-context AI actions: summarize, suggest, automate
- Predictive UI: suggesting next actions based on patterns
- **For Markplane**: AI context generation is core — surface this in the UI

### Other Notable Trends

- **Minimalism with personality**: asymmetric layouts, strategic color splashes, playful micro-interactions
- **Motion as design language**: cinematic transitions, not static screens
- **Accessibility regulation**: WCAG compliance becoming legally required — design for it from day one
- **Data density**: developer tools trending toward higher density with good hierarchy

---

## 5. Dark/Light Theme Best Practices

### Color System Architecture

**Semantic Token System:**
```
Semantic Token → Light Value / Dark Value
─────────────────────────────────────────
--background   → white / near-black
--surface      → light-gray / dark-gray
--elevated     → white / medium-gray
--border       → light-border / dark-border
--text-primary → near-black / near-white
--text-muted   → medium-gray / light-gray
```

Single semantic tokens automatically resolve based on mode — no separate light/dark token pairs in component code.

**Dark Theme Color Shifts:**
- Saturation decreases slightly in dark mode to reduce eye strain
- Background: never pure black (`#000`) — use 2-5% lightness for warmth
- Text: never pure white (`#FFF`) — use 95-98% lightness to reduce harshness
- Borders: use low-opacity white (`oklch(1 0 0 / 10-15%)`) rather than opaque gray

**The 60-30-10 Rule:**
- 60% dominant background color
- 30% secondary/surface color
- 10% accent colors (status, actions, interactive elements)

**Contrast Ratios (WCAG AA):**
- Normal text: 4.5:1 minimum
- Large text (18px+ or 14px+ bold): 3:1 minimum
- Interactive elements: 3:1 against adjacent colors

### Modern CSS Techniques

```css
/* Modern approach: prefers-color-scheme + light-dark() */
:root {
  color-scheme: light dark;
  --background: light-dark(oklch(1 0 0), oklch(0.145 0 0));
  --foreground: light-dark(oklch(0.145 0 0), oklch(0.985 0 0));
}
```

- `light-dark()` CSS function (2025+) simplifies dual-mode definitions
- `color-scheme` property enables native OS integration
- CSS custom properties with `.dark` class override remain most compatible approach

### LCH/OKLCH Color Space

- **Perceptually uniform**: equal numeric lightness differences appear equally different to human eyes
- Linear interpolation produces more natural gradients than HSL
- Adopted by: shadcn/ui, Vercel Geist, Linear
- Format: `oklch(lightness chroma hue)` — L: 0-1, C: saturation, H: 0-360

---

## 6. shadcn/ui Ecosystem

### Overview

shadcn/ui is the de facto standard for React/Next.js component libraries in 2025-2026. It provides accessible, themeable components built on **Radix primitives** with **Tailwind CSS** styling.

### Key Components for Markplane

| Component | Markplane Use Case |
|-----------|-------------------|
| **Sidebar** | Main navigation with collapsible state |
| **Data Table** | Task lists, backlog views (TanStack Table integration) |
| **Command** | Cmd+K command palette |
| **Card** | Task cards in kanban, dashboard widgets |
| **Badge** | Status labels, priority indicators, tags |
| **Dialog/Sheet** | Task detail view, quick actions |
| **Tabs** | View switching (kanban/list/timeline) |
| **Dropdown Menu** | Context menus, action menus |
| **Toast** | Operation feedback notifications |
| **Tooltip** | Hover information |
| **Chart** | Dashboard metrics (Recharts-based) |
| **Calendar** | Date pickers, timeline views |
| **Breadcrumb** | Navigation hierarchy |
| **Avatar** | User/assignee display |
| **Skeleton** | Loading states |

### Sidebar System

The sidebar is a first-class component with:
- **SidebarProvider** → manages collapsible state
- **Three variants**: `sidebar` (standard), `floating` (panel), `inset`
- **Three collapse modes**: `offcanvas`, `icon` (collapse to icons), `none`
- **Dedicated CSS variables**: `--sidebar-background`, `--sidebar-foreground`, `--sidebar-primary`, etc.
- **Keyboard shortcut**: Cmd+B toggle (configurable)
- **useSidebar hook**: `state`, `open`, `toggleSidebar()`, `isMobile`

### Theming System (Tailwind v4)

**CSS Variable Convention:**
- `--primary` = background color, `--primary-foreground` = text color on primary
- Variables defined in `:root` (light) and `.dark` (dark)

**Complete Variable Set:**
```css
:root {
  /* Backgrounds */
  --background: oklch(1 0 0);
  --foreground: oklch(0.145 0 0);

  /* Cards */
  --card: oklch(1 0 0);
  --card-foreground: oklch(0.145 0 0);

  /* Primary (buttons, links) */
  --primary: oklch(0.205 0 0);
  --primary-foreground: oklch(0.985 0 0);

  /* Secondary */
  --secondary: oklch(0.97 0 0);
  --secondary-foreground: oklch(0.205 0 0);

  /* Muted (disabled, placeholder) */
  --muted: oklch(0.97 0 0);
  --muted-foreground: oklch(0.556 0 0);

  /* Destructive (delete, error) */
  --destructive: oklch(0.577 0.245 27.325);

  /* Borders, inputs */
  --border: oklch(0.922 0 0);
  --input: oklch(0.922 0 0);
  --ring: oklch(0.708 0 0);

  /* Radius */
  --radius: 0.625rem;
}

.dark {
  --background: oklch(0.145 0 0);
  --foreground: oklch(0.985 0 0);
  --primary: oklch(0.922 0 0);
  --card: oklch(0.205 0 0);
  --border: oklch(1 0 0 / 10%);
  --input: oklch(1 0 0 / 15%);
  --destructive: oklch(0.704 0.191 22.216);
}
```

**Adding Custom Colors:**
```css
:root {
  --warning: oklch(0.84 0.16 84);
}
@theme inline {
  --color-warning: var(--warning);
}
```

### Tailwind v4 Integration

- `@theme` directive replaces `tailwind.config.js` theme extensions
- `@theme inline` for CSS variable-based colors
- `tw-animate-css` replaces deprecated `tailwindcss-animate`
- Full support for OKLCH color space
- CSS variables auto-generate utility classes: `bg-warning`, `text-warning`, etc.

### Data Table (TanStack Table)

- Built on `@tanstack/react-table` for sorting, filtering, pagination
- Column definitions with type safety
- Faceted filters, global search
- Row selection, bulk actions
- Virtual scrolling for large datasets

### Charts (Recharts)

- Five chart color variables (`--chart-1` through `--chart-5`)
- Bar, line, area, pie, radar chart types
- Responsive containers
- Custom tooltips with shadcn styling

### Blocks & Templates

- Pre-built dashboard layouts with sidebar navigation
- Kanban board blocks available in ecosystem (Shadcn UI Kit, Dice UI)
- Calendar, settings pages, authentication flows

---

## 7. Color Palette Recommendations for Markplane

### Dark Theme Base Palette

```
Background Layers (darkest to lightest):
──────────────────────────────────────
--bg-base:      oklch(0.13 0.005 260)   // #0C0D14 — deep space blue-black
--bg-surface:   oklch(0.17 0.005 260)   // #14151E — cards, panels
--bg-elevated:  oklch(0.21 0.005 260)   // #1C1D28 — hover states, dropdowns
--bg-overlay:   oklch(0.25 0.005 260)   // #252633 — modals, popovers

Border & Divider:
──────────────────
--border:       oklch(1 0 0 / 8%)       // subtle white borders
--border-hover: oklch(1 0 0 / 15%)      // interactive border state

Text:
──────
--text-primary:   oklch(0.95 0 0)       // near-white for headings
--text-secondary: oklch(0.70 0 0)       // gray for body text
--text-muted:     oklch(0.50 0 0)       // faded for metadata
```

### Light Theme Base Palette

```
Background Layers:
──────────────────
--bg-base:      oklch(0.985 0 0)        // off-white
--bg-surface:   oklch(0.97 0 0)         // light gray for cards
--bg-elevated:  oklch(1 0 0)            // pure white for elevated
--bg-overlay:   oklch(1 0 0)            // white modals

Border & Divider:
──────────────────
--border:       oklch(0 0 0 / 8%)       // subtle black borders
--border-hover: oklch(0 0 0 / 15%)      // interactive

Text:
──────
--text-primary:   oklch(0.15 0 0)       // near-black
--text-secondary: oklch(0.40 0 0)       // dark gray
--text-muted:     oklch(0.60 0 0)       // medium gray
```

### Semantic Status Colors

These colors should work on both dark and light backgrounds with appropriate lightness adjustments:

```
Status Colors (dark mode values):
─────────────────────────────────
--status-draft:       oklch(0.55 0.02 260)    // muted blue-gray
--status-backlog:     oklch(0.60 0.03 260)    // slightly brighter gray
--status-planned:     oklch(0.70 0.10 260)    // soft blue
--status-in-progress: oklch(0.72 0.15 250)    // vibrant cyan-blue
--status-done:        oklch(0.72 0.17 155)    // clean green
--status-blocked:     oklch(0.65 0.20 30)     // amber/orange
--status-cancelled:   oklch(0.55 0.02 0)      // muted gray-red
--status-archived:    oklch(0.45 0.01 260)    // very muted

Epic Statuses:
──────────────
--status-epic-planned: oklch(0.65 0.12 260)   // blue
--status-epic-active:  oklch(0.72 0.18 160)   // teal-green
--status-epic-done:    oklch(0.72 0.17 155)   // green (shared with task done)
```

### Priority Colors

```
Priority Colors (dark mode):
────────────────────────────
--priority-critical: oklch(0.65 0.25 25)    // red
--priority-high:     oklch(0.70 0.18 50)    // orange
--priority-medium:   oklch(0.78 0.15 85)    // yellow
--priority-low:      oklch(0.65 0.10 250)   // blue
--priority-none:     oklch(0.50 0.02 260)   // gray
```

### Accent Color

```
Brand Accent (Markplane Blue):
──────────────────────────────
--accent:           oklch(0.65 0.18 250)    // refined blue
--accent-hover:     oklch(0.70 0.20 250)    // lighter on hover
--accent-muted:     oklch(0.65 0.18 250 / 15%)  // subtle background tint
```

---

## 8. Component Design Patterns

### Kanban Board

**Best practices from research:**
- Drag states: idle → hover → grab → move → drop — each communicates visually
- Snap/ghost/bounce effects guide users during drag operations
- Column WIP limits with visual indicators
- Touch-friendly: larger drag zones, swipe-to-move fallback
- Keyboard accessible: arrow keys to move, space/enter to drop, escape to cancel
- WCAG 2.2 AAA: full screen reader announcements
- Column headers with count badges and color-coded status indicator
- Card design: title, priority badge, assignee avatar, tags, due date indicator

**Recommended library:** `@dnd-kit` or Dice UI's kanban component (shadcn-based)

### Timeline/Roadmap View

- Horizontal time axis with zoomable scales (day/week/month/quarter)
- Draggable bars for epics/tasks with dependency arrows
- Today marker line
- Swimlanes by epic or assignee

**Recommended:** SVAR React Gantt (MIT, React 19 compatible) or shadcn Gantt component

### Dependency Graph

- DAG (directed acyclic graph) visualization for `depends_on`/`blocks` relationships
- Hierarchical layout using Dagre algorithm (not force-directed — dependencies have inherent direction)
- Interactive: zoom, pan, node drag, hover details with entity preview
- Color-coded nodes by status, entity-colored borders by type
- Edge arrows showing dependency direction with animated flow indicators
- WikiLinkChip integration: `[[TASK-042]]` renders as clickable entity-colored chip

**Recommended:** `@xyflow/react` (React Flow) with Dagre layout — purpose-built for node-based DAGs, better than force-directed for dependency graphs

### Dashboard

- Grid layout with draggable/resizable widgets
- Key metrics: open tasks, blocked items, burndown, velocity
- Activity feed (recent changes)
- Quick actions (create task, start sprint)
- Status distribution charts (shadcn Chart components)

### Task Detail View

- Split or modal view (Linear-style side panel)
- Markdown body with live preview
- Frontmatter displayed as clean metadata fields
- Activity timeline / history
- Related items (dependencies, epic, plan)
- Quick status/priority/assignee dropdowns

### Command Palette

- Central hub: navigate to items, change status, create tasks, search
- Fuzzy matching with highlighting
- Grouped results (Tasks, Epics, Plans, Notes, Actions)
- Recent items / frequently used commands
- Keyboard shortcut hints alongside each action

**Recommended:** shadcn `Command` component (built on cmdk)

---

## 9. Design Inspiration Summary

### The Markplane Aesthetic

Based on this research, Markplane's web UI should embody:

1. **Linear's clarity**: Minimal chrome, high content density, keyboard-first interaction
2. **Vercel's restraint**: Pure, confident color palette — let typography and spacing do the work
3. **Obsidian's markdown soul**: Beautiful markdown rendering, wiki-link navigation, local-first feel
4. **Modern PM flexibility**: Multiple views (kanban, list, timeline, table) with smooth transitions

### Design DNA

```
Markplane = Linear's UI clarity
          + Vercel's dark theme refinement
          + Obsidian's markdown-first rendering
          + AI-native context awareness (unique)
```

### Key Differentiators for Markplane

1. **Markdown-native**: Content renders beautifully because the source IS markdown
2. **AI context layer**: `.context/` summaries visualized — show what the AI sees
3. **Filesystem transparency**: Users can see the actual file structure, not just abstracted views
4. **Git-native**: Activity history from git commits, not a database changelog
5. **Developer-first**: Monospaced fonts for IDs, code-editor-like markdown editing

### Recommended Font Stack

```css
--font-sans: 'Geist Sans', 'Inter', system-ui, -apple-system, sans-serif;
--font-mono: 'Geist Mono', 'JetBrains Mono', 'Fira Code', monospace;
--font-display: 'Inter Display', 'Geist Sans', system-ui, sans-serif;
```

### Recommended Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | Next.js 15 (App Router) |
| Styling | Tailwind CSS v4 |
| Components | shadcn/ui (Radix primitives) |
| Animations | Framer Motion |
| Charts | Recharts (via shadcn Chart) |
| Data Tables | TanStack Table (via shadcn Data Table) |
| Kanban DnD | @dnd-kit or Dice UI Kanban |
| Command Palette | cmdk (via shadcn Command) |
| Timeline | SVAR React Gantt or custom |
| Graph | @xyflow/react (React Flow) + Dagre |
| Theme | next-themes + shadcn theming |
| Icons | Lucide React |
| State | TanStack Query v5 (server state) + Zustand (client state if needed) |

---

## 10. Sources & References

### PM Tools
- [Linear UI Redesign](https://linear.app/now/how-we-redesigned-the-linear-ui)
- [Linear Style](https://linear.style/)
- [Linear Design Trend — LogRocket](https://blog.logrocket.com/ux-design/linear-design/)
- [Rise of Linear Style Design — Medium](https://medium.com/design-bootcamp/the-rise-of-linear-style-design-origins-trends-and-techniques-4fd96aab7646)
- [Plane.so](https://plane.so)
- [Huly Platform](https://github.com/hcengineering/platform)
- [Height App](https://height.app/)
- [Shortcut](https://www.getapp.com/project-management-planning-software/a/clubhouse/)

### Developer Dashboards
- [Vercel Dashboard UX — Medium](https://medium.com/design-bootcamp/vercels-new-dashboard-ux-what-it-teaches-us-about-developer-centric-design-93117215fe31)
- [Vercel Geist Design System](https://vercel.com/geist/introduction)
- [Geist Font](https://vercel.com/font)
- [Railway](https://railway.com)
- [Render](https://render.com)
- [GitHub Projects Docs](https://docs.github.com/en/issues/planning-and-tracking-with-projects)

### Design Trends
- [2026 Web Design Trends](https://www.digitalupward.com/blog/2026-web-design-trends-glassmorphism-micro-animations-ai-magic/)
- [UI Design Trends 2026](https://www.bookmarkify.io/blog/inspiration-ui-design)
- [8 UI Design Trends 2025](https://www.pixelmatters.com/insights/8-ui-design-trends-2025)
- [Command Palette Best Practices](https://solomon.io/designing-command-palettes/)
- [How to Build a Remarkable Command Palette — Superhuman](https://blog.superhuman.com/how-to-build-a-remarkable-command-palette/)
- [Command Palette — Mobbin](https://mobbin.com/glossary/command-palette)

### Theming & Color
- [Dark Mode Color Palettes Guide 2025](https://mypalettetool.com/blog/dark-mode-color-palettes)
- [Theming with CSS in 2025](https://mamutlove.com/en/blog/theming-with-css-in-2025/)
- [Color Tokens for Light/Dark Modes — Medium](https://medium.com/design-bootcamp/color-tokens-guide-to-light-and-dark-modes-in-design-systems-146ab33023ac)
- [Dark Mode Palette — Zeplin](https://blog.zeplin.io/design-delivery/dark-mode-color-palette/)

### shadcn/ui
- [shadcn/ui Theming](https://ui.shadcn.com/docs/theming)
- [shadcn/ui Tailwind v4](https://ui.shadcn.com/docs/tailwind-v4)
- [shadcn/ui Sidebar](https://ui.shadcn.com/docs/components/radix/sidebar)
- [shadcn/ui Data Table](https://ui.shadcn.com/docs/components/radix/data-table)
- [shadcn/ui Chart](https://ui.shadcn.com/docs/components/radix/chart)
- [shadcn/ui Blocks](https://ui.shadcn.com/blocks)
- [shadcn/ui Dark Mode](https://ui.shadcn.com/docs/dark-mode)

### Kanban & Visualization
- [Production-Ready Kanban with shadcn/ui](https://www.blog.brightcoding.dev/2025/12/12/%F0%9F%9A%80-the-ultimate-guide-to-building-a-production-ready-kanban-board-with-shadcn-ui-2025/)
- [Dice UI Kanban](https://www.diceui.com/docs/components/kanban)
- [Drag-and-Drop UI Best Practices — LogRocket](https://blog.logrocket.com/ux-design/drag-and-drop-ui-examples/)
- [SVAR React Gantt](https://svar.dev/react/gantt/)
- [shadcn Gantt](https://www.shadcn.io/components/data/gantt)
- [React Force Graph](https://github.com/vasturiano/react-force-graph)
