"use client";

import { useEffect, useRef, useMemo, useCallback, type ReactNode } from "react";
import { useRouter } from "next/navigation";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeSlug from "rehype-slug";
import { useTheme } from "next-themes";
import { PrismLight as SyntaxHighlighter } from "react-syntax-highlighter";
import { oneLight as oneLightRaw, oneDark as oneDarkRaw } from "react-syntax-highlighter/dist/esm/styles/prism";
import type { CSSProperties } from "react";
import bash from "react-syntax-highlighter/dist/esm/languages/prism/bash";
import json from "react-syntax-highlighter/dist/esm/languages/prism/json";
import yaml from "react-syntax-highlighter/dist/esm/languages/prism/yaml";
import markdown from "react-syntax-highlighter/dist/esm/languages/prism/markdown";
import rust from "react-syntax-highlighter/dist/esm/languages/prism/rust";
import typescript from "react-syntax-highlighter/dist/esm/languages/prism/typescript";
import toml from "react-syntax-highlighter/dist/esm/languages/prism/toml";
import type { Components } from "react-markdown";

SyntaxHighlighter.registerLanguage("bash", bash);
SyntaxHighlighter.registerLanguage("shell", bash);
SyntaxHighlighter.registerLanguage("sh", bash);
SyntaxHighlighter.registerLanguage("json", json);
SyntaxHighlighter.registerLanguage("yaml", yaml);
SyntaxHighlighter.registerLanguage("yml", yaml);
SyntaxHighlighter.registerLanguage("markdown", markdown);
SyntaxHighlighter.registerLanguage("md", markdown);
SyntaxHighlighter.registerLanguage("rust", rust);
SyntaxHighlighter.registerLanguage("typescript", typescript);
SyntaxHighlighter.registerLanguage("ts", typescript);
SyntaxHighlighter.registerLanguage("toml", toml);

/** Strip background colors from a react-syntax-highlighter theme so code blocks
 *  inherit the container's background (prose `<pre>` styling). */
function stripBackgrounds(theme: Record<string, CSSProperties>): Record<string, CSSProperties> {
  const result: Record<string, CSSProperties> = {};
  for (const [key, value] of Object.entries(theme)) {
    const { background, backgroundColor, ...rest } = value as CSSProperties & { background?: string; backgroundColor?: string };
    result[key] = rest;
  }
  return result;
}

const oneLight = stripBackgrounds(oneLightRaw);
const oneDark = stripBackgrounds(oneDarkRaw);

export interface Heading {
  id: string;
  text: string;
  level: number;
}

/** Build a regex that matches any of the given terms. Returns null if no terms. */
function buildTermsRegex(terms: string[], query: string): RegExp | null {
  // Use match terms from MiniSearch if available, otherwise fall back to raw query
  const patterns = terms.length > 0
    ? terms.map((t) => t.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"))
    : query
      ? [query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")]
      : [];
  if (patterns.length === 0) return null;
  return new RegExp(`(${patterns.join("|")})`, "gi");
}

function highlightMatch(text: string, regex: RegExp | null): ReactNode {
  if (!regex) return text;
  // Reset lastIndex since we reuse the regex
  regex.lastIndex = 0;
  const parts = text.split(regex);
  return parts.map((part, i) => {
    regex.lastIndex = 0;
    return regex.test(part) ? (
      <mark key={i} className="bg-yellow-200 dark:bg-yellow-500/40 text-foreground rounded-sm px-0.5">
        {part}
      </mark>
    ) : (
      part
    );
  });
}

function processChildren(children: ReactNode, regex: RegExp | null): ReactNode {
  if (!children || !regex) return children;
  if (typeof children === "string") {
    return highlightMatch(children, regex);
  }
  if (Array.isArray(children)) {
    return children.map((child, i) =>
      typeof child === "string" ? (
        <span key={i}>{highlightMatch(child, regex)}</span>
      ) : (
        child
      )
    );
  }
  return children;
}

interface DocRendererProps {
  content: string;
  searchQuery?: string;
  matchTerms?: string[];
  onHeadingsExtracted?: (headings: Heading[]) => void;
}

const remarkPlugins = [remarkGfm];
const rehypePlugins = [rehypeSlug];

/**
 * Extract a doc slug from an href if it points to a .md file in the docs.
 * Handles: "mcp-setup.md", "docs/web-ui-guide.md#settings", "/docs/file-format.md#anchor"
 * Returns { slug, anchor } or null if not a doc link.
 */
function parseDocHref(href: string): { slug: string; anchor: string } | null {
  const match = /^(?:\.\/)?(?:docs\/)?(?:\/docs\/)?([a-z0-9-]+)\.md(#[^\s)]*)?$/i.exec(href);
  if (!match) return null;
  return { slug: match[1], anchor: match[2] ?? "" };
}

export function DocRenderer({ content, searchQuery = "", matchTerms = [], onHeadingsExtracted }: DocRendererProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const router = useRouter();
  const { resolvedTheme } = useTheme();
  const syntaxTheme = resolvedTheme === "dark" ? oneDark : oneLight;

  const handleLinkClick = useCallback(
    (e: React.MouseEvent<HTMLAnchorElement>, href: string) => {
      const doc = parseDocHref(href);
      if (!doc) return; // Not a doc link — let browser handle it
      e.preventDefault();
      router.push(`/docs?page=${doc.slug}${doc.anchor}`, { scroll: false });
    },
    [router]
  );

  const termsRegex = useMemo(
    () => buildTermsRegex(matchTerms, searchQuery),
    [matchTerms, searchQuery]
  );

  // Extract headings after render
  useEffect(() => {
    if (!containerRef.current || !onHeadingsExtracted) return;
    const elements = containerRef.current.querySelectorAll("h1[id], h2[id], h3[id]");
    const headings: Heading[] = Array.from(elements).map((el) => ({
      id: el.id,
      text: el.textContent ?? "",
      level: parseInt(el.tagName[1]),
    }));
    onHeadingsExtracted(headings);
  }, [content, onHeadingsExtracted]);

  const components = useMemo<Components>(() => ({
    code: ({ className, children, ...props }) => {
      const match = /language-(\w+)/.exec(className ?? "");
      const codeString = String(children).replace(/\n$/, "");
      if (match) {
        return (
          <SyntaxHighlighter
            style={syntaxTheme}
            language={match[1]}
            customStyle={{ margin: 0, padding: "1em", fontSize: "0.875em" }}
          >
            {codeString}
          </SyntaxHighlighter>
        );
      }
      return (
        <code className={className} {...props}>
          {children}
        </code>
      );
    },
    // SyntaxHighlighter renders its own <pre>, so unwrap the outer one for
    // language-tagged blocks. For plain code blocks (no language), keep the <pre>.
    pre: ({ children, ...props }) => {
      const child = Array.isArray(children) ? children[0] : children;
      const hasLanguage = child?.props?.className && /language-\w+/.test(child.props.className);
      if (hasLanguage) return <>{children}</>;
      return (
        <pre {...props} className="not-prose overflow-x-auto rounded-md border bg-muted px-4 py-3 text-sm leading-relaxed font-mono">
          {children}
        </pre>
      );
    },
    a: ({ href, children, ...props }) => (
      <a href={href} onClick={(e) => href && handleLinkClick(e, href)} {...props}>
        {children}
      </a>
    ),
    p: ({ children, ...props }) => (
      <p {...props}>{processChildren(children, termsRegex)}</p>
    ),
    li: ({ children, ...props }) => (
      <li {...props}>{processChildren(children, termsRegex)}</li>
    ),
    td: ({ children, ...props }) => (
      <td {...props}>{processChildren(children, termsRegex)}</td>
    ),
    h1: ({ children, ...props }) => (
      <h1 {...props}>{processChildren(children, termsRegex)}</h1>
    ),
    h2: ({ children, ...props }) => (
      <h2 {...props}>{processChildren(children, termsRegex)}</h2>
    ),
    h3: ({ children, ...props }) => (
      <h3 {...props}>{processChildren(children, termsRegex)}</h3>
    ),
    h4: ({ children, ...props }) => (
      <h4 {...props}>{processChildren(children, termsRegex)}</h4>
    ),
  }), [termsRegex, syntaxTheme, handleLinkClick]);

  return (
    <div ref={containerRef} className="prose dark:prose-invert max-w-none prose-headings:text-foreground prose-h1:text-xl prose-h2:text-lg prose-h3:text-base prose-p:text-foreground/90 prose-a:text-primary prose-strong:text-foreground prose-code:text-foreground prose-code:bg-muted prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:text-sm prose-code:font-mono prose-code:border prose-code:border-border/50 prose-code:before:content-[''] prose-code:after:content-[''] prose-pre:bg-muted prose-pre:border prose-blockquote:border-primary/30 prose-li:marker:text-muted-foreground [&_pre_code]:border-0 [&_pre_code]:bg-transparent [&_pre_code]:p-0 [&_pre_code]:rounded-none">
      <ReactMarkdown
        remarkPlugins={remarkPlugins}
        rehypePlugins={rehypePlugins}
        components={components}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
}
