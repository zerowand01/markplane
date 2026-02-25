"use client";

import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import { CreateDialog } from "@/components/domain/create-dialog";

type EntityKind = "task" | "epic" | "note" | "plan";

const KIND_ROUTES: Record<EntityKind, (id: string) => string> = {
  task: (id) => `/backlog?task=${id}`,
  epic: (id) => `/roadmap?epic=${id}`,
  plan: (id) => `/plans?plan=${id}`,
  note: (id) => `/notes?note=${id}`,
};

export function GlobalCreateDialog() {
  const [open, setOpen] = useState(false);
  const [kind, setKind] = useState<EntityKind>("task");
  const router = useRouter();

  useEffect(() => {
    const handler = (e: Event) => {
      const detail = (e as CustomEvent).detail;
      if (detail?.kind) {
        setKind(detail.kind);
        setOpen(true);
      }
    };
    window.addEventListener("create-item", handler);
    return () => window.removeEventListener("create-item", handler);
  }, []);

  return (
    <CreateDialog
      kind={kind}
      open={open}
      onOpenChange={setOpen}
      onCreated={(id) => router.push(KIND_ROUTES[kind](id))}
    />
  );
}
