interface PlaybackControlsProps {
  isPlaying: boolean;
  currentPositionMs: number;
  durationMs: number;
  playbackSpeed: number;
  onPlayPause: () => void;
  onSeek: (positionMs: number) => void;
  onSpeedChange: (speed: number) => void;
}

function formatTime(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

export function PlaybackControls({
  isPlaying,
  currentPositionMs,
  durationMs,
  playbackSpeed,
  onPlayPause,
  onSeek,
  onSpeedChange,
}: PlaybackControlsProps) {
  function handleSpeedDecrease() {
    const newSpeed = Math.max(0.25, playbackSpeed - 0.05);
    onSpeedChange(Math.round(newSpeed * 100) / 100);
  }

  function handleSpeedIncrease() {
    const newSpeed = Math.min(1.0, playbackSpeed + 0.05);
    onSpeedChange(Math.round(newSpeed * 100) / 100);
  }

  function handleProgressClick(e: React.MouseEvent<HTMLDivElement>) {
    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const ratio = x / rect.width;
    onSeek(ratio * durationMs);
  }

  const progress = durationMs > 0 ? (currentPositionMs / durationMs) * 100 : 0;

  return (
    <div className="bg-bg-surface border-t border-border-subtle px-4 h-12 flex items-center gap-3 shrink-0">
      <button
        onClick={() => onSeek(Math.max(0, currentPositionMs - 5000))}
        className="bg-bg-input text-text-secondary w-8 h-8 flex items-center justify-center rounded-sm border border-border-subtle hover:text-text-primary hover:border-border-strong cursor-pointer transition-all text-body"
        aria-label="Retroceder 5 segundos"
      >
        ⏮
      </button>

      <button
        onClick={onPlayPause}
        className="bg-accent text-bg-root w-10 h-10 flex items-center justify-center rounded-sm hover:bg-accent-hover cursor-pointer transition-colors text-body font-semibold"
        aria-label={isPlaying ? "Pausar" : "Reproducir"}
      >
        {isPlaying ? "⏸" : "▶"}
      </button>

      <button
        onClick={() => onSeek(Math.min(durationMs, currentPositionMs + 5000))}
        className="bg-bg-input text-text-secondary w-8 h-8 flex items-center justify-center rounded-sm border border-border-subtle hover:text-text-primary hover:border-border-strong cursor-pointer transition-all text-body"
        aria-label="Adelantar 5 segundos"
      >
        ⏭
      </button>

      <span className="text-mono text-text-secondary bg-bg-input border border-border-subtle px-3 py-1 min-w-[60px] text-center">
        {formatTime(currentPositionMs)}
      </span>

      <div
        className="flex-1 h-2 bg-bg-input border border-border-subtle relative cursor-pointer rounded-sm"
        onClick={handleProgressClick}
      >
        <div
          className="absolute left-0 top-0 bottom-0 bg-accent rounded-sm"
          style={{ width: `${progress}%` }}
        />
      </div>

      <span className="text-mono text-text-secondary bg-bg-input border border-border-subtle px-3 py-1 min-w-[60px] text-center">
        {formatTime(durationMs)}
      </span>

      <div className="flex items-center gap-1.5 ml-2">
        <button
          onClick={handleSpeedDecrease}
          disabled={playbackSpeed <= 0.25}
          className="bg-bg-input text-text-secondary w-7 h-7 flex items-center justify-center rounded-sm border border-border-subtle hover:text-text-primary hover:border-border-strong cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed transition-all text-caption"
          aria-label="Reducir velocidad"
        >
          −
        </button>
        <span className="text-caption text-text-secondary min-w-[40px] text-center">
          <span className="text-accent">{Math.round(playbackSpeed * 100)}</span>%
        </span>
        <button
          onClick={handleSpeedIncrease}
          disabled={playbackSpeed >= 1.0}
          className="bg-bg-input text-text-secondary w-7 h-7 flex items-center justify-center rounded-sm border border-border-subtle hover:text-text-primary hover:border-border-strong cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed transition-all text-caption"
          aria-label="Aumentar velocidad"
        >
          +
        </button>
      </div>
    </div>
  );
}
