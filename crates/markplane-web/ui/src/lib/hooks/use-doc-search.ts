"use client";

import { useMemo, useCallback } from "react";
import MiniSearch from "minisearch";
import type { DocContent } from "@/lib/types";

interface DocSection {
  id: string;
  docSlug: string;
  docTitle: string;
  heading: string;
  anchor: string;
  body: string;
  level: number;
}

export interface DocSearchResult {
  id: string;
  docSlug: string;
  docTitle: string;
  heading: string;
  anchor: string;
  excerpt: string;
  score: number;
}

export interface GroupedSearchResults {
  docSlug: string;
  docTitle: string;
  results: DocSearchResult[];
}

/** Slugify a heading to match rehype-slug output. */
function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^\w\s-]/g, "")
    .replace(/\s+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "");
}

/** Strip markdown formatting from text for plain-text indexing. */
function stripMarkdown(text: string): string {
  return text
    .replace(/```[\s\S]*?```/g, "")       // fenced code blocks
    .replace(/`[^`]+`/g, "")              // inline code
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1") // links → text
    .replace(/[#*_~|>]/g, "")             // formatting chars
    .replace(/\s+/g, " ")
    .trim();
}

/** Split a markdown doc into sections by headings. */
function parseMarkdownSections(slug: string, title: string, content: string): DocSection[] {
  const lines = content.split("\n");
  const sections: DocSection[] = [];
  const anchorCounts = new Map<string, number>();
  let currentHeading = title;
  let currentAnchor = "";
  let currentLevel = 1;
  let bodyLines: string[] = [];

  /** Deduplicate anchors to match rehype-slug behavior (appends -1, -2, etc.). */
  function dedupeAnchor(anchor: string): string {
    const count = anchorCounts.get(anchor) ?? 0;
    anchorCounts.set(anchor, count + 1);
    return count === 0 ? anchor : `${anchor}-${count}`;
  }

  function flush() {
    const body = stripMarkdown(bodyLines.join("\n"));
    if (body) {
      const id = currentAnchor ? `${slug}#${currentAnchor}` : slug;
      sections.push({
        id,
        docSlug: slug,
        docTitle: title,
        heading: currentHeading,
        anchor: currentAnchor,
        body,
        level: currentLevel,
      });
    }
    bodyLines = [];
  }

  for (const line of lines) {
    const match = /^(#{1,3})\s+(.+)$/.exec(line);
    if (match) {
      flush();
      currentLevel = match[1].length;
      currentHeading = match[2].replace(/[#*_`]/g, "").trim();
      currentAnchor = dedupeAnchor(slugify(currentHeading));
    } else {
      bodyLines.push(line);
    }
  }
  flush();

  return sections;
}

/** Generate a ~150 char excerpt around the first match of any query term. */
function generateExcerpt(body: string, terms: string[]): string {
  const lower = body.toLowerCase();
  let bestPos = -1;
  for (const term of terms) {
    const pos = lower.indexOf(term.toLowerCase());
    if (pos !== -1 && (bestPos === -1 || pos < bestPos)) {
      bestPos = pos;
    }
  }
  if (bestPos === -1) return body.slice(0, 150).trim() + (body.length > 150 ? "..." : "");

  const start = Math.max(0, bestPos - 40);
  const end = Math.min(body.length, bestPos + 110);
  let excerpt = body.slice(start, end).trim();
  if (start > 0) excerpt = "..." + excerpt;
  if (end < body.length) excerpt = excerpt + "...";
  return excerpt;
}

export function useDocSearch(allContents: DocContent[] | undefined) {
  const miniSearch = useMemo(() => {
    if (!allContents || allContents.length === 0) return null;

    const sections = allContents.flatMap((doc) =>
      parseMarkdownSections(doc.slug, doc.title, doc.content)
    );

    const ms = new MiniSearch<DocSection>({
      fields: ["heading", "body", "docTitle"],
      storeFields: ["docSlug", "docTitle", "heading", "anchor", "body", "level"],
      searchOptions: {
        boost: { heading: 5, docTitle: 3 },
        prefix: true,
        fuzzy: 0.2,
      },
    });
    ms.addAll(sections);
    return ms;
  }, [allContents]);

  const search = useCallback(
    (query: string): GroupedSearchResults[] => {
      if (!miniSearch || !query.trim()) return [];

      const raw = miniSearch.search(query);
      if (raw.length === 0) return [];

      // Collect matched terms for excerpt generation
      const allTerms = new Set<string>();
      for (const r of raw) {
        for (const t of Object.keys(r.match)) allTerms.add(t);
      }
      const terms = Array.from(allTerms);

      // Map to our result type
      const results: DocSearchResult[] = raw.map((r) => ({
        id: r.id as string,
        docSlug: r.docSlug as string,
        docTitle: r.docTitle as string,
        heading: r.heading as string,
        anchor: r.anchor as string,
        excerpt: generateExcerpt(r.body as string, terms),
        score: r.score,
      }));

      // Group by doc, preserving score order within groups
      const groups = new Map<string, GroupedSearchResults>();
      for (const result of results) {
        let group = groups.get(result.docSlug);
        if (!group) {
          group = { docSlug: result.docSlug, docTitle: result.docTitle, results: [] };
          groups.set(result.docSlug, group);
        }
        group.results.push(result);
      }

      // Sort groups by best score (highest first)
      return Array.from(groups.values()).sort(
        (a, b) => b.results[0].score - a.results[0].score
      );
    },
    [miniSearch]
  );

  const getMatchTerms = useCallback(
    (query: string): string[] => {
      if (!miniSearch || !query.trim()) return [];
      const raw = miniSearch.search(query);
      const terms = new Set<string>();
      for (const r of raw) {
        for (const t of Object.keys(r.match)) terms.add(t);
      }
      return Array.from(terms);
    },
    [miniSearch]
  );

  return { search, getMatchTerms };
}
