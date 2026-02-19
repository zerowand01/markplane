import { Node, mergeAttributes } from "@tiptap/core";

/**
 * TipTap node for [[WIKI-LINK]] syntax.
 *
 * Wiki-links are semantic entities, not plain text. Without a dedicated node type,
 * they're stored as text nodes and prosemirror-markdown's serializer escapes the
 * brackets: [[TASK-001]] becomes \[\[TASK-001\]\].
 *
 * This extension solves it at the right layer:
 * - Parse: markdown-it inline rule tokenizes [[PREFIX-NNN]] into a custom token,
 *   rendered as <span data-wiki-link="ID"> for TipTap's HTML parser
 * - Render: displayed as a styled inline chip in the rich editor
 * - Serialize: state.write() outputs [[ID]] directly, bypassing text escaping
 */

const WIKI_LINK_RE = /\[\[((?:TASK|EPIC|PLAN|NOTE)-\d{3,})\]\]/;

const PREFIX_COLORS: Record<string, string> = {
  TASK: "var(--entity-task)",
  EPIC: "var(--entity-epic)",
  PLAN: "var(--entity-plan)",
  NOTE: "var(--entity-note)",
};

export const WikiLink = Node.create({
  name: "wikiLink",
  group: "inline",
  inline: true,
  atom: true,
  selectable: true,
  draggable: false,

  addAttributes() {
    return {
      id: {
        default: null,
        parseHTML: (element) => element.getAttribute("data-wiki-link"),
        renderHTML: (attributes) => ({ "data-wiki-link": attributes.id }),
      },
    };
  },

  parseHTML() {
    return [{ tag: "span[data-wiki-link]" }];
  },

  renderHTML({ node, HTMLAttributes }) {
    const id = node.attrs.id as string;
    const prefix = id?.split("-")[0] ?? "TASK";
    const color = PREFIX_COLORS[prefix] ?? PREFIX_COLORS.TASK;

    return [
      "span",
      mergeAttributes(HTMLAttributes, {
        class: "wiki-link-chip",
        style: [
          "display: inline-flex",
          "align-items: center",
          "font-family: monospace",
          "font-size: 0.75rem",
          "padding: 1px 6px",
          "border-radius: 4px",
          `background-color: color-mix(in oklch, ${color} 15%, transparent)`,
          `color: ${color}`,
        ].join("; "),
      }),
      id,
    ];
  },

  addStorage() {
    return {
      markdown: {
        serialize(state: any, node: any) {
          state.write(`[[${node.attrs.id}]]`);
        },
        parse: {
          setup(markdownit: any) {
            markdownit.inline.ruler.push(
              "wiki_link",
              (state: any, silent: boolean) => {
                const src = state.src.slice(state.pos);
                const match = src.match(WIKI_LINK_RE);

                if (!match || match.index !== 0) return false;
                if (silent) return true;

                const token = state.push("wiki_link", "", 0);
                token.content = match[1];
                state.pos += match[0].length;

                return true;
              }
            );

            markdownit.renderer.rules.wiki_link = (
              tokens: any[],
              idx: number
            ) => {
              const id = tokens[idx].content;
              return `<span data-wiki-link="${id}">${id}</span>`;
            };
          },
        },
      },
    };
  },
});
