---
id: NOTE-kbtk5
title: 'ADR: Client-only rendering for page components via dynamic ssr:false'
status: active
type: decision
related:
- TASK-ax3dv
tags:
- web-ui
- architecture
- next-js
created: 2026-02-25
updated: 2026-02-25
---

# ADR: Client-only rendering for page components via dynamic ssr:false

## Status

Accepted (2026-02-25)

## Context

All 7 content pages (dashboard, backlog, roadmap, plans, notes, graph, archive) used `<Suspense fallback={<Skeleton />}>` wrapping a content component that called `useSearchParams()`. This caused React hydration mismatch errors:

- During SSR (Turbopack dev) or static generation (`next build`), `useSearchParams()` suspends because URL params aren't available. React renders the Suspense fallback (skeleton) into HTML.
- On the client, `useSearchParams()` resolves synchronously. The content component renders — either a skeleton (if TanStack Query cache is empty) or actual content (if cache is warm from HMR or prior navigation).
- When the client's first render differs from the server HTML, React reports a hydration mismatch and regenerates the entire tree.

The error was "recoverable" (React re-renders on the client) but caused unnecessary full re-renders and console errors on every page load in development.

### Approaches investigated and rejected

1. **Lifting** `PageTransition` **above** `Suspense` — addressed one structural inconsistency (the animation wrapper) but didn't fix the fundamental mismatch between Suspense fallback and client render. Tested and failed.

2. **Server Component pages with Client Component content files** — the Next.js-recommended pattern for `useSearchParams`. Made `page.tsx` a Server Component with `<Suspense>` wrapping an imported Client Component. Built successfully but the hydration error persisted — SSR still renders a skeleton that can mismatch the client when TanStack Query has cached data.

3. `useHydrated` **/** `ClientOnly` **pattern** — gates rendering until after `useEffect` fires, ensuring server and client both render the skeleton. Works but is a workaround that masks the issue. Adds an unnecessary extra render frame and doesn't address the architectural mismatch.

## Decision

Use `next/dynamic` with `ssr: false` for all page content components. Each page is split into two files:

- `page.tsx` — thin `"use client"` shell with a `dynamic()` import and skeleton loading fallback
- `*-content.tsx` — all page logic, hooks, state, and rendering

```tsx
// page.tsx
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

The content component is never server-rendered. Both server and client initially show the same skeleton, then the client lazily loads and renders the content component. No hydration mismatch is possible.

## Rationale

This is not a workaround — it's the architecturally correct choice for this app:

1. **Markplane is a local-first client-only app.** With `output: 'export'`, there is no server at runtime. The Rust binary serves static files and a REST API. All meaningful content comes from client-side API calls (`/api/tasks`, `/api/epics`, etc.) that cannot execute at build time.

2. **SSR provides zero value here.** Server-rendering these components can only ever produce a loading skeleton — the data doesn't exist during build or SSR. The skeleton is then immediately replaced on the client. SSR adds complexity (Suspense boundaries, hydration coordination) for no user-visible benefit.

3. `ssr: false` **declares intent.** It explicitly states "this component is client-only," which is the truth. The components depend on browser APIs (`useSearchParams`, `window.location`), client-side state (TanStack Query cache), and a runtime API server.

4. **Precedent in the codebase.** The graph page already used `dynamic` with `ssr: false` for its `GraphView` component for the same reason (heavy client-only component with no server-side value).

## Trade-offs

- **Code splitting**: Each content component becomes a separate JS chunk. On localhost this is imperceptible. The `loading` fallback covers the gap.
- **File count**: 7 pages become 14 files (7 shells + 7 content). The shells are 13-17 lines each. The separation is clean and improves organization.
- **No Suspense boundaries**: The `Suspense` + `useSearchParams` pattern is removed. `useSearchParams` runs purely on the client, so no Suspense wrapper is needed.

## References

- \[\[TASK-ax3dv\]\] — the bug this decision resolves
- [Next.js useSearchParams docs](https://nextjs.org/docs/app/api-reference/functions/use-search-params#static-rendering)
- [Next.js deopted into client rendering](https://nextjs.org/docs/messages/deopted-into-client-rendering)
