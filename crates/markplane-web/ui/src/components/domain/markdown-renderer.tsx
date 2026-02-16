"use client";

import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { WikiLinkChip } from "./wiki-link-chip";
import type { ReactNode } from "react";

const WIKI_LINK_REGEX = /\[\[(TASK|EPIC|PLAN|NOTE)-\d{3,}\]\]/g;

function processWikiLinks(text: string): ReactNode[] {
  const parts: ReactNode[] = [];
  let lastIndex = 0;

  for (const match of text.matchAll(WIKI_LINK_REGEX)) {
    const id = match[0].slice(2, -2); // Remove [[ and ]]
    if (match.index! > lastIndex) {
      parts.push(text.slice(lastIndex, match.index));
    }
    parts.push(<WikiLinkChip key={`${id}-${match.index}`} id={id} />);
    lastIndex = match.index! + match[0].length;
  }
  if (lastIndex < text.length) {
    parts.push(text.slice(lastIndex));
  }
  return parts;
}

export function MarkdownRenderer({ content }: { content: string }) {
  return (
    <div className="prose dark:prose-invert max-w-none prose-headings:text-foreground prose-h1:text-xl prose-h2:text-lg prose-h3:text-base prose-p:text-foreground/90 prose-a:text-primary prose-strong:text-foreground prose-code:text-foreground prose-code:bg-muted prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:text-sm prose-pre:bg-muted prose-pre:border prose-blockquote:border-primary/30 prose-li:marker:text-muted-foreground">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          p: ({ children, ...props }) => (
            <p {...props}>{processChildren(children)}</p>
          ),
          li: ({ children, ...props }) => (
            <li {...props}>{processChildren(children)}</li>
          ),
          td: ({ children, ...props }) => (
            <td {...props}>{processChildren(children)}</td>
          ),
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
}

function processChildren(children: ReactNode): ReactNode {
  if (!children) return children;
  if (typeof children === "string") {
    const parts = processWikiLinks(children);
    return parts.length === 1 ? parts[0] : <>{parts}</>;
  }
  if (Array.isArray(children)) {
    return children.map((child, i) =>
      typeof child === "string" ? (
        <span key={i}>{processWikiLinks(child)}</span>
      ) : (
        child
      )
    );
  }
  return children;
}
