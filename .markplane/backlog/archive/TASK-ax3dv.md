---
id: TASK-ax3dv
title: Fix hydration mismatch on pages using Suspense with PageTransition
status: done
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

During SSR (Turbopack dev) or static generation (`next build`), `useSearchParams()` suspends because URL params aren't available at build time. React renders the Suspense fallback (skeleton) into HTML. On the client, `useSearchParams()` resolves synchronously and the content component renders — either a skeleton (empty TanStack Query cache) or actual content (warm cache from HMR or prior navigation). When the client's first render differs from the server HTML, React reports a hydration mismatch.

The PageTransition wrapper was an additional structural inconsistency (server HTML never had the `animate-in` div, client always did), but fixing that alone did not resolve the error. The fundamental issue is that SSR provides no value for this app — all data comes from client-side API calls to the local Rust server, so server-rendering can only ever produce a loading skeleton.

### Affected pages

All 7 content pages: backlog, dashboard, roadmap, plans, notes, graph, archive.

## Fix (implemented)

Replaced `<Suspense>` boundaries with `next/dynamic` + `ssr: false`. Each page was split into two files:

- **`page.tsx`** — thin `"use client"` shell with `dynamic()` import and skeleton loading fallback
- **`*-content.tsx`** — all page logic moved from the original file

```tsx
// page.tsx (example: backlog)
"use client";
import dynamic from "next/dynamic";
import { BacklogSkeleton } from "./backlog-content";

const BacklogContent = dynamic(
  () => import("./backlog-content").then((m) => ({ default: m.BacklogContent })),
  { ssr: false, loading: () => <BacklogSkeleton /> }
);

export default function BacklogPage() {
  return (
    <div className="p-4 md:p-6">
      <BacklogContent />
    </div>
  );
}
```

The content component is never server-rendered. Both server and client initially show the same skeleton, eliminating any possible hydration mismatch. See [[NOTE-kbtk5]] for full architectural decision record.

### Approaches investigated and rejected

1. **Lifting PageTransition above Suspense** — addressed one structural inconsistency but didn't fix the fundamental SSR/client mismatch. Tested, failed.
2. **Server Component pages** — made page.tsx a Server Component with Suspense wrapping a Client Component content file. Built but hydration error persisted.
3. **`useHydrated` pattern** — workaround that delays rendering until after useEffect. Masks the issue without addressing the architecture.

## Acceptance Criteria

- [x] No hydration mismatch error on any page (verified backlog, roadmap, dashboard, graph, plans, notes, archive)
- [x] All 7 pages split into `page.tsx` (dynamic shell) + `*-content.tsx` (client code)
- [x] `next build` passes cleanly with all 11 routes as static content
- [x] Page entrance animations still work correctly
