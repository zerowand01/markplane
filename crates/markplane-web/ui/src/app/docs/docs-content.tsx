"use client";

import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import { Search, Menu, X, ChevronUp, ChevronDown } from "lucide-react";
import { useDocs, useDoc, useAllDocContents } from "@/lib/hooks/use-docs";
import { useIsMobile } from "@/hooks/use-mobile";
import { useDocSearch } from "@/lib/hooks/use-doc-search";
import { DocRenderer, type Heading } from "./doc-renderer";

export function DocsSkeleton() {
  return (
    <div className="flex h-[calc(100vh-3.5rem)] md:h-screen">
      <div className="hidden md:block w-64 border-r p-4 space-y-3">
        <div className="h-8 bg-muted rounded animate-pulse" />
        {Array.from({ length: 7 }).map((_, i) => (
          <div key={i} className="h-6 bg-muted rounded animate-pulse" />
        ))}
      </div>
      <div className="flex-1 p-8 space-y-4">
        <div className="h-8 w-64 bg-muted rounded animate-pulse" />
        <div className="h-4 w-full bg-muted rounded animate-pulse" />
        <div className="h-4 w-3/4 bg-muted rounded animate-pulse" />
        <div className="h-4 w-5/6 bg-muted rounded animate-pulse" />
      </div>
    </div>
  );
}

function TocPanel({ headings, activeId }: { headings: Heading[]; activeId: string }) {
  if (headings.length === 0) return null;

  return (
    <nav className="space-y-1">
      <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">
        Contents
      </p>
      {headings.map((heading) => (
        <a
          key={heading.id}
          href={`#${heading.id}`}
          onClick={(e) => {
            e.preventDefault();
            document.getElementById(heading.id)?.scrollIntoView({ behavior: "smooth" });
          }}
          className={`block text-sm truncate transition-colors ${
            heading.level === 2 ? "pl-0" : heading.level === 3 ? "pl-3" : "pl-0"
          } ${
            activeId === heading.id
              ? "text-primary font-medium"
              : "text-muted-foreground hover:text-foreground"
          }`}
        >
          {heading.text}
        </a>
      ))}
    </nav>
  );
}

export function DocsContent() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const isMobile = useIsMobile();
  const contentRef = useRef<HTMLDivElement>(null);

  const { data: docs, isLoading: isLoadingDocs } = useDocs();
  const { data: allContents } = useAllDocContents();

  const selectedSlug = searchParams.get("page") ?? docs?.[0]?.slug ?? "";
  const { data: docContent, isLoading: isLoadingDoc } = useDoc(selectedSlug, {
    enabled: !!selectedSlug,
  });

  const [searchQuery, setSearchQuery] = useState("");
  const [headings, setHeadings] = useState<Heading[]>([]);
  const [activeHeadingId, setActiveHeadingId] = useState("");
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [currentMatchIndex, setCurrentMatchIndex] = useState(0);
  const pendingAnchorRef = useRef<string | null>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);

  // Refs to break stale closure chains in navigation callbacks
  const currentMatchIndexRef = useRef(0);
  const selectedSlugRef = useRef(selectedSlug);
  const allResultsRef = useRef<import("@/lib/hooks/use-doc-search").DocSearchResult[]>([]);

  const { search, getMatchTerms } = useDocSearch(allContents);

  // Compute search results and match terms
  const searchResults = useMemo(
    () => (searchQuery ? search(searchQuery) : []),
    [searchQuery, search]
  );

  const matchTerms = useMemo(
    () => (searchQuery ? getMatchTerms(searchQuery) : []),
    [searchQuery, getMatchTerms]
  );

  // Flat list of all section results for navigation
  const allResults = useMemo(
    () => searchResults.flatMap((g) => g.results),
    [searchResults]
  );

  // Keep refs in sync
  useEffect(() => {
    currentMatchIndexRef.current = currentMatchIndex;
    selectedSlugRef.current = selectedSlug;
    allResultsRef.current = allResults;
  }, [currentMatchIndex, selectedSlug, allResults]);

  // Docs that have search results (preserves original order)
  const filteredDocs = useMemo(() => {
    if (!docs) return [];
    if (!searchQuery || searchResults.length === 0) return docs;
    const slugsWithResults = new Set(searchResults.map((g) => g.docSlug));
    return docs.filter((doc) => slugsWithResults.has(doc.slug));
  }, [docs, searchQuery, searchResults]);

  // Reset match index when query changes (state-during-render pattern)
  const [prevSearchQuery, setPrevSearchQuery] = useState(searchQuery);
  if (prevSearchQuery !== searchQuery) {
    setPrevSearchQuery(searchQuery);
    setCurrentMatchIndex(0);
  }

  // Stable navigation — reads from refs so no stale closures
  const navigateTo = useCallback(
    (slug: string, anchor?: string) => {
      if (slug === selectedSlugRef.current) {
        if (anchor) {
          document.getElementById(anchor)?.scrollIntoView({ behavior: "smooth" });
        } else {
          contentRef.current?.scrollTo({ top: 0, behavior: "smooth" });
        }
      } else {
        if (anchor) pendingAnchorRef.current = anchor;
        router.push(`/docs?page=${slug}`, { scroll: false });
      }
      if (isMobile) setSidebarOpen(false);
    },
    [router, isMobile]
  );

  const goToResult = useCallback(
    (index: number) => {
      const result = allResultsRef.current[index];
      if (!result) return;
      setCurrentMatchIndex(index);
      navigateTo(result.docSlug, result.anchor || undefined);
    },
    [navigateTo]
  );

  const nextMatch = useCallback(() => {
    const len = allResultsRef.current.length;
    if (len === 0) return;
    goToResult((currentMatchIndexRef.current + 1) % len);
  }, [goToResult]);

  const prevMatch = useCallback(() => {
    const len = allResultsRef.current.length;
    if (len === 0) return;
    goToResult((currentMatchIndexRef.current - 1 + len) % len);
  }, [goToResult]);

  const handleHeadingsExtracted = useCallback((h: Heading[]) => {
    setHeadings(h);
    // Consume pending anchor after content has rendered (headings are in the DOM)
    const anchor = pendingAnchorRef.current;
    if (anchor) {
      pendingAnchorRef.current = null;
      requestAnimationFrame(() => {
        document.getElementById(anchor)?.scrollIntoView({ behavior: "smooth" });
      });
    }
  }, []);

  // Scroll to top when doc changes (without a pending anchor)
  useEffect(() => {
    if (!pendingAnchorRef.current) {
      contentRef.current?.scrollTo(0, 0);
    }
  }, [selectedSlug]);

  // Handle keyboard shortcuts in search input
  const handleSearchKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter" && allResults.length > 0) {
        e.preventDefault();
        if (e.shiftKey) {
          prevMatch();
        } else {
          nextMatch();
        }
      } else if (e.key === "Escape") {
        setSearchQuery("");
        searchInputRef.current?.blur();
      }
    },
    [allResults.length, nextMatch, prevMatch]
  );

  // Track active heading by scroll position — find the last heading
  // that has scrolled past the top of the viewport (with a small offset).
  useEffect(() => {
    const container = contentRef.current;
    if (!container || headings.length === 0) return;

    const OFFSET = 100; // px below container top to consider "active"

    function onScroll() {
      const containerTop = container!.getBoundingClientRect().top;
      let activeId = headings[0]?.id ?? "";
      for (const heading of headings) {
        const el = document.getElementById(heading.id);
        if (!el) continue;
        const elTop = el.getBoundingClientRect().top - containerTop;
        if (elTop <= OFFSET) {
          activeId = heading.id;
        } else {
          break;
        }
      }
      setActiveHeadingId(activeId);
    }

    onScroll(); // Set initial state
    container.addEventListener("scroll", onScroll, { passive: true });
    return () => container.removeEventListener("scroll", onScroll);
  }, [headings]);

  const sidebarContent = (
    <div className="flex flex-col h-full">
      <div className="p-3 border-b">
        <div className="relative">
          <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
          <input
            ref={searchInputRef}
            type="text"
            placeholder="Search docs..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onKeyDown={handleSearchKeyDown}
            className={`w-full pl-8 ${searchQuery ? "pr-8" : "pr-3"} py-1.5 text-sm rounded-md border bg-background focus:outline-none focus:ring-1 focus:ring-ring`}
          />
          {searchQuery && (
            <button
              onClick={() => { setSearchQuery(""); searchInputRef.current?.focus(); }}
              className="absolute right-2 top-1/2 -translate-y-1/2 p-0.5 rounded-sm text-muted-foreground hover:text-foreground"
            >
              <X className="size-3.5" />
            </button>
          )}
        </div>
        {searchQuery && allResults.length > 0 && (
          <div className="flex items-center justify-between mt-2 text-xs text-muted-foreground">
            <span>{allResults.length} result{allResults.length !== 1 ? "s" : ""}</span>
            <div className="flex items-center gap-1">
              <button
                onClick={prevMatch}
                className="p-0.5 rounded hover:bg-accent"
                title="Previous (Shift+Enter)"
              >
                <ChevronUp className="size-3.5" />
              </button>
              <button
                onClick={nextMatch}
                className="p-0.5 rounded hover:bg-accent"
                title="Next (Enter)"
              >
                <ChevronDown className="size-3.5" />
              </button>
            </div>
          </div>
        )}
      </div>
      <nav className="flex-1 overflow-y-auto p-2">
        {isLoadingDocs ? (
          <div className="space-y-2 p-2">
            {Array.from({ length: 7 }).map((_, i) => (
              <div key={i} className="h-6 bg-muted rounded animate-pulse" />
            ))}
          </div>
        ) : searchQuery && searchResults.length > 0 ? (
          // Show grouped section results when searching
          <div className="space-y-3">
            {searchResults.map((group) => (
              <div key={group.docSlug}>
                <button
                  onClick={() => navigateTo(group.docSlug)}
                  className={`w-full text-left px-3 py-1 text-sm font-medium transition-colors rounded-md ${
                    group.docSlug === selectedSlug
                      ? "text-accent-foreground"
                      : "text-foreground hover:text-foreground"
                  }`}
                >
                  {group.docTitle}
                </button>
                <ul className="mt-0.5 space-y-0.5">
                  {group.results.map((result) => (
                    <li key={result.id}>
                      <button
                        onClick={() => navigateTo(result.docSlug, result.anchor)}
                        className="w-full text-left px-3 py-1.5 rounded-md text-xs transition-colors hover:bg-accent/50 group"
                      >
                        <span className="text-muted-foreground group-hover:text-foreground block truncate">
                          {result.heading}
                        </span>
                        <span className="text-muted-foreground/70 block truncate mt-0.5">
                          {result.excerpt}
                        </span>
                      </button>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </div>
        ) : (
          // Default doc list
          <ul className="space-y-0.5">
            {filteredDocs.map((doc) => (
              <li key={doc.slug}>
                <button
                  onClick={() => navigateTo(doc.slug)}
                  className={`w-full text-left px-3 py-1.5 rounded-md text-sm transition-colors ${
                    doc.slug === selectedSlug
                      ? "bg-accent text-accent-foreground font-medium"
                      : "text-muted-foreground hover:text-foreground hover:bg-accent/50"
                  }`}
                >
                  <span className="truncate">{doc.title}</span>
                </button>
              </li>
            ))}
          </ul>
        )}
      </nav>
    </div>
  );

  return (
    <div className="flex h-[calc(100vh-3.5rem)] md:h-screen overflow-hidden">
      {/* Mobile sidebar toggle */}
      {isMobile && (
        <button
          onClick={() => setSidebarOpen(!sidebarOpen)}
          className="fixed top-14 left-3 z-40 size-8 flex items-center justify-center rounded-md bg-background border shadow-sm"
        >
          {sidebarOpen ? <X className="size-4" /> : <Menu className="size-4" />}
        </button>
      )}

      {/* Mobile sidebar overlay */}
      {isMobile && sidebarOpen && (
        <>
          <div
            className="fixed inset-0 bg-black/20 z-30"
            onClick={() => setSidebarOpen(false)}
          />
          <div className="fixed left-0 top-14 bottom-0 w-64 bg-background border-r z-30">
            {sidebarContent}
          </div>
        </>
      )}

      {/* Desktop sidebar */}
      {!isMobile && (
        <div className="w-64 border-r shrink-0">
          {sidebarContent}
        </div>
      )}

      {/* Content area */}
      <div ref={contentRef} className="flex-1 overflow-y-auto">
        <div className="max-w-3xl mx-auto px-6 py-8 md:px-8">
          {isLoadingDoc ? (
            <div className="space-y-4">
              <div className="h-8 w-64 bg-muted rounded animate-pulse" />
              <div className="h-4 w-full bg-muted rounded animate-pulse" />
              <div className="h-4 w-3/4 bg-muted rounded animate-pulse" />
              <div className="h-4 w-5/6 bg-muted rounded animate-pulse" />
            </div>
          ) : docContent ? (
            <DocRenderer
              content={docContent.content}
              searchQuery={searchQuery}
              matchTerms={matchTerms}
              onHeadingsExtracted={handleHeadingsExtracted}
            />
          ) : (
            <p className="text-muted-foreground">Select a document from the sidebar.</p>
          )}
        </div>
      </div>

      {/* TOC panel — desktop only (>=1280px) */}
      <div className="hidden xl:block w-56 border-l shrink-0 overflow-y-auto p-4">
        <TocPanel headings={headings} activeId={activeHeadingId} />
      </div>
    </div>
  );
}
