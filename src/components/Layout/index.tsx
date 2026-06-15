import { ReactNode } from "react";
import { PlayerBar } from "./PlayerBar";

interface LayoutProps {
  children: ReactNode;
  showPlayerBar?: boolean;
}

export function Layout({ children, showPlayerBar }: LayoutProps) {
  return (
    <div className="h-screen bg-bg-root text-text-primary flex flex-col overflow-hidden">
      {showPlayerBar && <PlayerBar />}
      <main className="flex-1 flex flex-col overflow-hidden">{children}</main>
    </div>
  );
}
