interface TapCalibrationPanelProps {
  detectedBpm: number | null;
  tapCount: number;
  acceptedCount: number;
  rejectedCount: number;
  canCommit: boolean;
  onTap: () => void;
  onCommit: () => void;
  onCancel: () => void;
}

export function TapCalibrationPanel({
  detectedBpm,
  tapCount,
  acceptedCount,
  rejectedCount,
  canCommit,
  onTap,
  onCommit,
  onCancel,
}: TapCalibrationPanelProps) {
  return (
    <div className="mt-2 pt-2 border-t border-border-subtle flex flex-col gap-2">
      <div className="flex items-center gap-2">
        <button
          onPointerDown={onTap}
          className="h-10 w-10 rounded-sm bg-accent text-bg-root flex items-center justify-center
                     active:scale-96 transition-transform duration-50 cursor-pointer select-none shrink-0"
          aria-label="Registrar pulso de calibración"
        >
          <span className="text-caption font-bold select-none">TAP</span>
        </button>

        <div className="flex flex-col min-w-0">
          <span className="text-body font-bold font-mono text-accent">
            {detectedBpm !== null ? `${detectedBpm.toFixed(1)} BPM` : "--.- BPM"}
          </span>
          <span className="text-caption text-text-tertiary">
            {tapCount === 0
              ? "Presiona T o TAP al ritmo"
              : `${tapCount} pulso${tapCount !== 1 ? "s" : ""}`}
            {rejectedCount > 0 && ` (${acceptedCount} ok, ${rejectedCount} desc.)`}
          </span>
        </div>
      </div>

      {tapCount > 0 && tapCount < 8 && (
        <span className="text-xs text-text-tertiary">
          Mínimo 8 pulsaciones para confirmar ({8 - tapCount} restantes)
        </span>
      )}

      <div className="flex gap-2">
        <button
          onClick={onCommit}
          disabled={!canCommit}
          className="flex-1 h-6 text-caption rounded-sm cursor-pointer transition-all
                     bg-accent text-bg-root disabled:opacity-30 disabled:cursor-not-allowed"
        >
          Confirmar
        </button>
        <button
          onClick={onCancel}
          className="flex-1 h-6 text-caption rounded-sm cursor-pointer transition-all
                     bg-bg-input text-text-tertiary hover:text-text-primary border border-border-subtle"
        >
          Cancelar
        </button>
      </div>
    </div>
  );
}
