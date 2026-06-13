import { ReactNode } from "react";

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  return (
    <div className="h-screen bg-bg-root text-text-primary flex flex-col overflow-hidden">
      <header className="bg-bg-surface border-b border-border-subtle px-4 h-10 flex items-center shrink-0">
        <div className="flex items-center justify-between w-full max-w-[1400px] mx-auto">
          <h1 className="text-sm font-semibold tracking-tight text-text-primary">Bassical</h1>
          <span className="text-caption text-text-tertiary">v0.1.0</span>
        </div>
      </header>
      <main className="flex-1 flex flex-col overflow-hidden">{children}</main>
    </div>
  );
}
