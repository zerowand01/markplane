import * as React from "react"

const MOBILE_BREAKPOINT = 768
const QUERY = `(max-width: ${MOBILE_BREAKPOINT - 1}px)`

// One shared MediaQueryList, created lazily on the client so the module is
// safe to import during prerender. Snapshot and subscription read the same
// source, so they can never disagree.
let mql: MediaQueryList | undefined
function mediaQuery() {
  return (mql ??= window.matchMedia(QUERY))
}

function subscribe(onChange: () => void) {
  const m = mediaQuery()
  m.addEventListener("change", onChange)
  return () => m.removeEventListener("change", onChange)
}

function getSnapshot() {
  return mediaQuery().matches
}

// No window during prerender/hydration; match the previous default of "not mobile".
function getServerSnapshot() {
  return false
}

export function useIsMobile() {
  return React.useSyncExternalStore(subscribe, getSnapshot, getServerSnapshot)
}
