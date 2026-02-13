"use client";

import { Component, type ReactNode } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { AlertTriangle } from "lucide-react";

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback;

      return (
        <Card className="border-destructive/30">
          <CardContent className="flex flex-col items-center justify-center gap-4 py-12">
            <AlertTriangle className="size-8 text-destructive" />
            <div className="text-center space-y-1">
              <p className="text-sm font-medium">Something went wrong</p>
              <p className="text-xs text-muted-foreground max-w-md">
                {this.state.error?.message || "An unexpected error occurred."}
              </p>
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={() => this.setState({ hasError: false, error: null })}
            >
              Try again
            </Button>
          </CardContent>
        </Card>
      );
    }

    return this.props.children;
  }
}
