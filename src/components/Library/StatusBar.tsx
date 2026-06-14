interface StatusBarProps {
  totalCount: number;
  missingCount: number;
}

export function StatusBar({ totalCount, missingCount }: StatusBarProps) {
  return (
    <div className="bg-bg-surface border-t border-border-subtle px-4 h-7 flex items-center justify-between shrink-0">
      <div className="flex items-center gap-4">
        <span className="text-caption text-text-secondary">
          {totalCount} {totalCount === 1 ? "canción" : "canciones"}
        </span>
        {missingCount > 0 && (
          <span className="text-caption text-text-danger flex items-center gap-1.5">
            <span className="inline-block w-1.5 h-1.5 rounded-full bg-text-danger" />
            {missingCount} {missingCount === 1 ? "archivo faltante" : "archivos faltantes"}
          </span>
        )}
      </div>
      <span className="text-caption text-text-tertiary">Bassical</span>
    </div>
  );
}
