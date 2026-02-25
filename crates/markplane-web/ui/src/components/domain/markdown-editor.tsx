"use client";

import { useState, useCallback, useRef, useEffect } from "react";
import { Extension } from "@tiptap/core";
import { useEditor, EditorContent } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import TaskList from "@tiptap/extension-task-list";
import TaskItem from "@tiptap/extension-task-item";
import Link from "@tiptap/extension-link";
import { Markdown } from "tiptap-markdown";
import { WikiLink } from "./tiptap-wiki-link";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import {
  Bold,
  Italic,
  Strikethrough,
  Heading1,
  Heading2,
  Heading3,
  Heading4,
  List,
  ListOrdered,
  ListChecks,
  Code,
  CodeXml,
  Quote,
  Minus,
  Link as LinkIcon,
  ChevronDown,
  Undo,
  Redo,
  Pilcrow,
} from "lucide-react";

/**
 * tiptap-markdown's MarkdownTightLists adds the `tight` attribute to bulletList
 * and orderedList, but omits taskList. Without it, the serializer treats task
 * lists as loose (blank lines between items). This extension fills that gap.
 */
const TaskListTight = Extension.create({
  name: "taskListTight",
  addGlobalAttributes() {
    return [
      {
        types: ["taskList"],
        attributes: {
          tight: {
            default: true,
            parseHTML: (element: HTMLElement) =>
              element.getAttribute("data-tight") === "true" ||
              !element.querySelector("p"),
            renderHTML: (attributes: Record<string, unknown>) => ({
              "data-tight": attributes.tight ? "true" : null,
            }),
          },
        },
      },
    ];
  },
});

interface MarkdownEditorProps {
  content: string;
  onSave: (markdown: string) => void;
  onCancel: () => void;
  isLoading?: boolean;
}

export function MarkdownEditor({
  content,
  onSave,
  onCancel,
  isLoading,
}: MarkdownEditorProps) {
  const [mode, setMode] = useState<"rich" | "source">("rich");
  const [sourceContent, setSourceContent] = useState(content);

  const editor = useEditor({
    immediatelyRender: false,
    extensions: [
      StarterKit,
      TaskList,
      TaskItem.configure({ nested: true }),
      TaskListTight,
      Link.configure({
        openOnClick: false,
        HTMLAttributes: { class: "text-primary underline" },
      }),
      WikiLink,
      Markdown.configure({
        html: false,
        transformPastedText: true,
        transformCopiedText: true,
      }),
    ],
    content,
    editorProps: {
      attributes: {
        class:
          "prose dark:prose-invert max-w-none prose-headings:text-foreground prose-h1:text-xl prose-h2:text-lg prose-h3:text-base prose-p:text-foreground/90 prose-a:text-primary prose-strong:text-foreground prose-code:text-foreground prose-code:bg-muted prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:text-sm prose-code:font-mono prose-code:border prose-code:border-border/50 prose-code:before:content-[''] prose-code:after:content-[''] prose-pre:bg-muted prose-pre:border prose-blockquote:border-primary/30 prose-li:marker:text-muted-foreground min-h-[120px] outline-none px-3 py-2",
      },
    },
  });

  const getMarkdown = useCallback(() => {
    if (mode === "source") {
      return sourceContent;
    }
    if (!editor) return content;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return (editor.storage as any).markdown.getMarkdown() as string;
  }, [mode, sourceContent, editor, content]);

  const handleSave = () => {
    onSave(getMarkdown());
  };

  const switchToSource = () => {
    if (editor) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      setSourceContent((editor.storage as any).markdown.getMarkdown());
    }
    setMode("source");
  };

  const switchToRich = () => {
    if (editor) {
      editor.commands.setContent(sourceContent);
    }
    setMode("rich");
  };

  return (
    <div className="border rounded-md">
      {/* Sticky toolbar — stays visible when scrolling the sheet */}
      <div className="sticky top-0 z-10 bg-card rounded-t-md">
        {/* Header: mode toggle + actions */}
        <div className="flex items-center justify-between border-b px-2 py-1.5 bg-muted/30 rounded-t-md">
          <div className="flex gap-1">
            <Button
              variant={mode === "rich" ? "secondary" : "ghost"}
              size="xs"
              onClick={mode === "source" ? switchToRich : undefined}
            >
              Rich text
            </Button>
            <Button
              variant={mode === "source" ? "secondary" : "ghost"}
              size="xs"
              onClick={mode === "rich" ? switchToSource : undefined}
            >
              Markdown
            </Button>
          </div>
          <div className="flex gap-1.5">
            <Button variant="ghost" size="sm" onClick={onCancel}>
              Cancel
            </Button>
            <Button
              size="sm"
              onClick={handleSave}
              disabled={isLoading}
            >
              Save
            </Button>
          </div>
        </div>

        {/* Formatting toolbar (rich mode only) */}
        {mode === "rich" && editor && (
          <div className="flex items-center gap-0.5 border-b px-2 py-1 bg-muted/30 flex-wrap">
            <ToolbarButton
              onClick={() => editor.chain().focus().toggleBold().run()}
              active={editor.isActive("bold")}
              title="Bold"
            >
              <Bold className="size-4" />
            </ToolbarButton>
            <ToolbarButton
              onClick={() => editor.chain().focus().toggleItalic().run()}
              active={editor.isActive("italic")}
              title="Italic"
            >
              <Italic className="size-4" />
            </ToolbarButton>
            <ToolbarButton
              onClick={() => editor.chain().focus().toggleStrike().run()}
              active={editor.isActive("strike")}
              title="Strikethrough"
            >
              <Strikethrough className="size-4" />
            </ToolbarButton>
            <span className="w-px h-5 bg-border mx-1" />
            <HeadingDropdown editor={editor} />
            <ToolbarButton
              onClick={() => editor.chain().focus().toggleBulletList().run()}
              active={editor.isActive("bulletList")}
              title="Bullet list"
            >
              <List className="size-4" />
            </ToolbarButton>
            <ToolbarButton
              onClick={() =>
                editor.chain().focus().toggleOrderedList().run()
              }
              active={editor.isActive("orderedList")}
              title="Ordered list"
            >
              <ListOrdered className="size-4" />
            </ToolbarButton>
            <ToolbarButton
              onClick={() => editor.chain().focus().toggleTaskList().run()}
              active={editor.isActive("taskList")}
              title="Task list"
            >
              <ListChecks className="size-4" />
            </ToolbarButton>
            <span className="w-px h-5 bg-border mx-1" />
            <ToolbarButton
              onClick={() => editor.chain().focus().toggleCode().run()}
              active={editor.isActive("code")}
              title="Inline code"
            >
              <Code className="size-4" />
            </ToolbarButton>
            <ToolbarButton
              onClick={() => editor.chain().focus().toggleCodeBlock().run()}
              active={editor.isActive("codeBlock")}
              title="Code block"
            >
              <CodeXml className="size-4" />
            </ToolbarButton>
            <ToolbarButton
              onClick={() =>
                editor.chain().focus().toggleBlockquote().run()
              }
              active={editor.isActive("blockquote")}
              title="Blockquote"
            >
              <Quote className="size-4" />
            </ToolbarButton>
            <ToolbarButton
              onClick={() =>
                editor.chain().focus().setHorizontalRule().run()
              }
              title="Horizontal rule"
            >
              <Minus className="size-4" />
            </ToolbarButton>
            <span className="w-px h-5 bg-border mx-1" />
            <ToolbarButton
              onClick={() => {
                const url = window.prompt("URL:");
                if (url) {
                  editor
                    .chain()
                    .focus()
                    .extendMarkRange("link")
                    .setLink({ href: url })
                    .run();
                }
              }}
              active={editor.isActive("link")}
              title="Link"
            >
              <LinkIcon className="size-4" />
            </ToolbarButton>
            <span className="w-px h-5 bg-border mx-1" />
            <ToolbarButton
              onClick={() => editor.chain().focus().undo().run()}
              disabled={!editor.can().undo()}
              title="Undo"
            >
              <Undo className="size-4" />
            </ToolbarButton>
            <ToolbarButton
              onClick={() => editor.chain().focus().redo().run()}
              disabled={!editor.can().redo()}
              title="Redo"
            >
              <Redo className="size-4" />
            </ToolbarButton>
          </div>
        )}
      </div>

      {/* Editor content */}
      {mode === "rich" ? (
        <EditorContent editor={editor} />
      ) : (
        <Textarea
          value={sourceContent}
          onChange={(e) => setSourceContent(e.target.value)}
          className="border-0 rounded-none shadow-none focus-visible:ring-0 min-h-[200px] font-mono text-sm resize-y"
        />
      )}
    </div>
  );
}

const HEADING_OPTIONS = [
  { level: 1 as const, label: "Heading 1", icon: Heading1 },
  { level: 2 as const, label: "Heading 2", icon: Heading2 },
  { level: 3 as const, label: "Heading 3", icon: Heading3 },
  { level: 4 as const, label: "Heading 4", icon: Heading4 },
] as const;

function HeadingDropdown({
  editor,
}: {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  editor: any;
}) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handleClick = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, [open]);

  const activeHeading = HEADING_OPTIONS.find((h) =>
    editor.isActive("heading", { level: h.level })
  );

  const ActiveIcon = activeHeading?.icon ?? Pilcrow;

  return (
    <div className="relative" ref={ref}>
      <button
        type="button"
        onClick={() => setOpen(!open)}
        title="Heading level"
        className={`flex items-center gap-0.5 p-1.5 rounded cursor-pointer transition-colors ${
          activeHeading
            ? "bg-accent text-accent-foreground"
            : "hover:bg-accent/50 text-muted-foreground hover:text-foreground"
        }`}
      >
        <ActiveIcon className="size-4" />
        <ChevronDown className="size-3" />
      </button>
      {open && (
        <div className="absolute top-full left-0 mt-1 bg-popover border rounded-md shadow-md z-20 py-1 min-w-[150px]">
          {HEADING_OPTIONS.map((h) => {
            const Icon = h.icon;
            return (
              <button
                key={h.level}
                type="button"
                onClick={() => {
                  editor
                    .chain()
                    .focus()
                    .toggleHeading({ level: h.level })
                    .run();
                  setOpen(false);
                }}
                className={`flex items-center gap-2 w-full px-3 py-1.5 text-sm cursor-pointer transition-colors ${
                  editor.isActive("heading", { level: h.level })
                    ? "bg-accent text-accent-foreground"
                    : "hover:bg-accent/50 text-foreground"
                }`}
              >
                <Icon className="size-4" />
                {h.label}
              </button>
            );
          })}
          <div className="h-px bg-border my-1" />
          <button
            type="button"
            onClick={() => {
              editor.chain().focus().setParagraph().run();
              setOpen(false);
            }}
            className={`flex items-center gap-2 w-full px-3 py-1.5 text-sm cursor-pointer transition-colors ${
              !activeHeading
                ? "bg-accent text-accent-foreground"
                : "hover:bg-accent/50 text-foreground"
            }`}
          >
            <Pilcrow className="size-4" />
            Normal text
          </button>
        </div>
      )}
    </div>
  );
}

function ToolbarButton({
  children,
  onClick,
  active,
  disabled,
  title,
}: {
  children: React.ReactNode;
  onClick?: () => void;
  active?: boolean;
  disabled?: boolean;
  title?: string;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      title={title}
      className={`p-1.5 rounded cursor-pointer transition-colors ${
        active
          ? "bg-accent text-accent-foreground"
          : "hover:bg-accent/50 text-muted-foreground hover:text-foreground"
      } ${disabled ? "opacity-30 cursor-not-allowed" : ""}`}
    >
      {children}
    </button>
  );
}
