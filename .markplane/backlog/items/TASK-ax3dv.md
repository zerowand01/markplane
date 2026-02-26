---
id: TASK-ax3dv
title: Fix hydration mismatch on pages using Suspense with PageTransition
status: backlog
priority: high
type: bug
effort: medium
tags:
- web-ui
epic: null
plan: null
depends_on: []
blocks: []
assignee: null
position: a0
created: 2026-02-25
updated: 2026-02-25
---

# Fix hydration mismatch on pages using Suspense with PageTransition

## Description

React hydration error on multiple pages: "Hydration failed because the server rendered HTML didn't match the client." The error is recoverable (React re-renders the tree on the client) but causes an unnecessary full re-render and console error.

### Root cause

Content components have inconsistent tree structures depending on loading state. Two variants of the same underlying problem:

**Variant A — Early return bypasses PageTransition** (backlog, dashboard):
- **Loading path** (early return): returns `<Skeleton />` directly — no `PageTransition` wrapper
- **Loaded path**: returns `<PageTransition><div className="space-y-4">...</div></PageTransition>`
- Triggers when `useSearchParams()` suspends during SSR (renders fallback) but resolves on the client with a warm TanStack Query cache (skips isLoading, renders loaded path with PageTransition wrapper)

**Variant B — Suspense fallback lacks PageTransition** (roadmap, graph, plans, notes):
- **Suspense fallback** (server HTML): renders `<Skeleton />` without `PageTransition` wrapper
- **Content component**: wraps ALL return paths (including loading) in `<PageTransition>`
- So server HTML never has the `<div class="animate-in...">` wrapper, but client always does
- Triggers on every page load where `useSearchParams()` suspends during SSR — more severe than Variant A

**Archive — latent structural inconsistency** (archive):
- Same structure as Variant A, but `ArchiveContent` does NOT call `useSearchParams()`, so Suspense never actually suspends during SSR
- Server HTML comes from the `isLoading` early return (not the Suspense fallback), and the client also starts with `isLoading: true` (empty cache), so tree matches during hydration
- Not an active bug but has the same structural inconsistency — should be fixed for consistency and to prevent regressions if `useSearchParams()` is added later

### Affected pages

| Page | File | Variant | Severity |
|------|------|---------|----------|
| **Roadmap** | `roadmap/page.tsx` | B | Active — triggers on every load |
| **Graph** | `graph/page.tsx` | B | Active — triggers on every load |
| **Plans** | `plans/page.tsx` | B | Active — triggers on every load |
| **Notes** | `notes/page.tsx` | B | Active — triggers on every load |
| **Backlog** | `backlog/page.tsx` | A | Active — triggers when TQ cache is warm |
| **Dashboard** | `dashboard/page.tsx` | A | Active — triggers when TQ cache is warm |
| **Archive** | `archive/page.tsx` | Latent | Structural only — no useSearchParams suspension |

These are all 7 Suspense pages. The remaining pages (root redirect) have no Suspense boundaries.

## Fix

`PageTransition` is a page-level presentation concern (entrance animation). It should be lifted above `Suspense` in the page component, not inside the content component. This ensures the wrapper is always present regardless of Suspense or loading state.

For each affected page:

1. Move `<PageTransition>` from inside the content component to the page-level component, wrapping the `<Suspense>` boundary
2. Remove the `<PageTransition>` wrapper from the content component's loaded return
3. The early `isLoading` return (Variant A) stays as-is — it now naturally matches the Suspense fallback since both produce the same HTML without a PageTransition wrapper (PageTransition is above both)

**Before** (backlog — Variant A):
```tsx
// BacklogPage
<Suspense fallback={<BacklogSkeleton />}>
  <BacklogContent />     // ← wraps loaded return in PageTransition
</Suspense>
```

**After**:
```tsx
// BacklogPage
<PageTransition>
  <Suspense fallback={<BacklogSkeleton />}>
    <BacklogContent />   // ← no PageTransition, consistent tree
  </Suspense>
</PageTransition>
```

**Before** (roadmap — Variant B):
```tsx
// RoadmapPage
<Suspense fallback={<RoadmapSkeleton />}>  // ← no PageTransition
  <RoadmapContent />     // ← wraps in PageTransition internally
</Suspense>
```

**After**:
```tsx
// RoadmapPage
<PageTransition>
  <Suspense fallback={<RoadmapSkeleton />}>
    <RoadmapContent />   // ← no PageTransition, consistent tree
  </Suspense>
</PageTransition>
```

This ensures server HTML, Suspense fallback, loading state, and loaded state all share the same outer tree prefix.

### Notes on the fix

- **Error early returns** in Variant B pages (roadmap, graph, plans, notes) currently bypass PageTransition. After the fix, they'll be inside PageTransition from the page level — this is fine and more consistent.
- **Entrance animation on fallback**: With PageTransition above Suspense, the skeleton fallback will also get the entrance animation. This is better UX (smooth slide-in for the skeleton).
- **No re-animation on content swap**: `animate-in` is a one-shot CSS animation that plays when the element mounts. When Suspense swaps fallback → content, the PageTransition div is already mounted, so the animation doesn't replay.

## Acceptance Criteria

- [ ] No hydration mismatch error on any page (verify backlog, roadmap, dashboard, graph, plans, notes)
- [ ] PageTransition lifted to page level on all 7 Suspense pages (backlog, dashboard, archive, roadmap, graph, plans, notes)
- [ ] PageTransition removed from inside all 7 content components
- [ ] Page entrance animations still work correctly
