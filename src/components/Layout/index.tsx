import { ReactNode } from "react";

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  return (
    <div className="min-h-screen bg-gray-900 text-white">
      <header className="bg-gray-800 p-4 border-b border-gray-700">
        <div className="container mx-auto flex items-center justify-between">
          <h1 className="text-xl font-bold text-white">Bassical</h1>
          <span className="text-sm text-gray-400">v0.1.0</span>
        </div>
      </header>
      <main className="container mx-auto p-4">{children}</main>
    </div>
  );
}
