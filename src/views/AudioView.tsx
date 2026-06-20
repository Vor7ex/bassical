import type { Song } from "@/lib/types";
import { usePracticePlayback } from "@/lib/usePracticePlayback";
import { WaveformView, PlaybackControls } from "@/components/Audio";

interface AudioViewProps {
  song: Song;
  onBack: () => void;
}

export function AudioView({ song, onBack }: AudioViewProps) {
  const {
    isPlaying,
    playbackSpeed,
    currentPositionMs,
    audioState,
    decodeProgress,
    fullBufferReady,
    handlePlayPause,
    handleSeek,
    handleSpeedChange,
  } = usePracticePlayback(song.audioPath);

  return (
    <div className="flex flex-col h-full">
      <div className="bg-bg-surface border-b border-border-subtle px-4 h-10 flex items-center justify-between shrink-0">
        <div className="flex items-center gap-3">
          <button
            onClick={onBack}
            className="bg-bg-input text-text-secondary px-3 h-7 text-caption rounded-sm border border-border-subtle hover:text-text-primary hover:border-border-strong cursor-pointer transition-all"
          >
            ← Biblioteca
          </button>
          <div className="w-px h-5 bg-border-subtle" />
          <span className="text-body text-text-primary font-semibold">
            {song.title}
          </span>
          {song.artist && (
            <span className="text-caption text-text-secondary">
              {song.artist}
            </span>
          )}
        </div>
        <span className="text-mono text-text-tertiary text-caption">
          {audioState
            ? `${Math.floor(audioState.durationMs / 60000)}:${String(
                Math.floor((audioState.durationMs % 60000) / 1000),
              ).padStart(2, "0")}`
            : "--:--"}
        </span>
      </div>

      <div className="flex-1 flex flex-col overflow-hidden p-4 gap-4">
        {!audioState ? (
          <div className="flex-1 flex items-center justify-center">
            <span className="text-body text-text-tertiary">
              Cargando audio...
            </span>
          </div>
        ) : audioState.peaks.length === 0 ? (
          <div className="flex-1 flex items-center justify-center">
            <span className="text-body text-text-tertiary">
              Decodificando audio...
            </span>
          </div>
        ) : (
          <WaveformView
            peaks={audioState.peaks}
            currentPositionMs={currentPositionMs}
            durationMs={audioState.durationMs}
            onSeek={handleSeek}
          />
        )}
      </div>

      <PlaybackControls
        isPlaying={isPlaying}
        currentPositionMs={currentPositionMs}
        durationMs={audioState?.durationMs ?? 0}
        playbackSpeed={playbackSpeed}
        speedDisabled={!fullBufferReady}
        onPlayPause={handlePlayPause}
        onSeek={handleSeek}
        onSpeedChange={handleSpeedChange}
      />

      <div className="bg-bg-surface border-t border-border-subtle px-4 h-7 flex items-center justify-between shrink-0">
        <div className="flex items-center gap-4">
          <span className="text-caption text-text-secondary">
            Velocidad:{" "}
            <span className="text-accent">
              {Math.round(playbackSpeed * 100)}
            </span>
            %
          </span>
          {decodeProgress < 1.0 ? (
            <span className="text-caption text-text-tertiary flex items-center gap-2">
              <span className="inline-block w-24 h-1.5 bg-bg-input rounded-full overflow-hidden">
                <span
                  className="block h-full bg-accent rounded-full transition-all duration-200"
                  style={{ width: `${Math.round(decodeProgress * 100)}%` }}
                />
              </span>
              Decodificando: {Math.round(decodeProgress * 100)}%
            </span>
          ) : (
            <span className="text-caption text-text-tertiary">
              Timing points: 0
            </span>
          )}
        </div>
        <span className="text-caption text-text-tertiary">Bassical</span>
      </div>
    </div>
  );
}
