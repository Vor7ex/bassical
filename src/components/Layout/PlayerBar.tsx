import { useEffect, useRef } from "react";
import { useSessionStore, useLibraryStore } from "@/lib/store";
import { useAudioPlayback } from "@/lib/useAudioPlayback";
import { ActiveTasksIndicator } from "./ActiveTasksIndicator";

const formatTime = (ms: number) => {
  if (!ms || isNaN(ms)) return "0:00";
  const totalSeconds = Math.floor(ms / 1000);
  const m = Math.floor(totalSeconds / 60);
  const s = String(totalSeconds % 60).padStart(2, "0");
  return `${m}:${s}`;
};

function useAutoPlayNext(
  audioState: { durationMs: number } | null,
  currentPositionMs: number,
  playNext: () => void,
) {
  const autoNextTriggeredRef = useRef(false);
  useEffect(() => {
    const isFinished =
      audioState &&
      currentPositionMs > 0 &&
      currentPositionMs >= audioState.durationMs - 100;
    if (isFinished && !autoNextTriggeredRef.current) {
      autoNextTriggeredRef.current = true;
      playNext();
    } else if (!isFinished) {
      autoNextTriggeredRef.current = false;
    }
  }, [currentPositionMs, audioState, playNext]);
}

function PlaybackControls({
  isPlaying,
  isLooping,
  isShuffle,
  hasSong,
  onPlayPause,
  onPrevious,
  onNext,
  onToggleLoop,
  onToggleShuffle,
}: {
  isPlaying: boolean;
  isLooping: boolean;
  isShuffle: boolean;
  hasSong: boolean;
  onPlayPause: () => void;
  onPrevious: () => void;
  onNext: () => void;
  onToggleLoop: () => void;
  onToggleShuffle: () => void;
}) {
  return (
    <div className="flex items-center gap-4">
      <button
        onClick={onToggleShuffle}
        className={`p-1.5 rounded-sm transition-colors cursor-pointer ${
          isShuffle ? "text-accent" : "text-text-tertiary hover:text-text-secondary"
        }`}
        title="Aleatorio"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="16 3 21 3 21 8"></polyline><line x1="4" y1="20" x2="21" y2="3"></line><polyline points="21 16 21 21 16 21"></polyline><line x1="15" y1="15" x2="21" y2="21"></line><line x1="4" y1="4" x2="9" y2="9"></line></svg>
      </button>
      <button
        onClick={onPrevious}
        disabled={!hasSong}
        className="p-1.5 text-text-secondary hover:text-text-primary disabled:opacity-50 transition-colors cursor-pointer"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="currentColor" stroke="none"><polygon points="19 20 9 12 19 4 19 20"></polygon><line x1="5" y1="19" x2="5" y2="5" stroke="currentColor" strokeWidth="2" strokeLinecap="round"></line></svg>
      </button>
      <button
        onClick={onPlayPause}
        disabled={!hasSong}
        className="w-8 h-8 flex items-center justify-center rounded-full bg-text-primary text-bg-surface hover:scale-105 disabled:opacity-50 disabled:hover:scale-100 transition-all cursor-pointer"
      >
        {isPlaying ? (
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" stroke="none"><rect x="6" y="4" width="4" height="16"></rect><rect x="14" y="4" width="4" height="16"></rect></svg>
        ) : (
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" stroke="none" className="ml-1"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
        )}
      </button>
      <button
        onClick={onNext}
        disabled={!hasSong}
        className="p-1.5 text-text-secondary hover:text-text-primary disabled:opacity-50 transition-colors cursor-pointer"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="currentColor" stroke="none"><polygon points="5 4 15 12 5 20 5 4"></polygon><line x1="19" y1="5" x2="19" y2="19" stroke="currentColor" strokeWidth="2" strokeLinecap="round"></line></svg>
      </button>
      <button
        onClick={onToggleLoop}
        className={`p-1.5 rounded-sm transition-colors cursor-pointer ${
          isLooping ? "text-accent" : "text-text-tertiary hover:text-text-secondary"
        }`}
        title="Repetir"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="17 1 21 5 17 9"></polyline><path d="M3 11V9a4 4 0 0 1 4-4h14"></path><polyline points="7 23 3 19 7 15"></polyline><path d="M21 13v2a4 4 0 0 1-4 4H3"></path></svg>
      </button>
    </div>
  );
}

function SongInfo({
  song,
}: {
  song: { title: string; artist: string | null } | null;
}) {
  if (!song) {
    return (
      <span className="text-caption text-text-tertiary">
        Ninguna canción seleccionada
      </span>
    );
  }
  return (
    <div className="flex flex-col min-w-0">
      <span className="text-body text-text-primary font-medium truncate">
        {song.title}
      </span>
      <span className="text-caption text-text-secondary truncate">
        {song.artist || "Desconocido"}
      </span>
    </div>
  );
}

export function PlayerBar() {
  const currentSongId = useSessionStore((s) => s.currentSongId);
  const isLooping = useSessionStore((s) => s.isLooping);
  const isShuffle = useSessionStore((s) => s.isShuffle);
  const toggleLoop = useSessionStore((s) => s.toggleLoop);
  const toggleShuffle = useSessionStore((s) => s.toggleShuffle);
  const playNext = useSessionStore((s) => s.playNext);
  const playPrevious = useSessionStore((s) => s.playPrevious);
  
  const songs = useLibraryStore((s) => s.songs);
  const song = songs.find((s) => s.id === currentSongId) ?? null;

  const {
    isPlaying,
    currentPositionMs,
    audioState,
    handlePlayPause,
    handleSeek,
  } = useAudioPlayback(song?.audioPath ?? "");

  useAutoPlayNext(audioState, currentPositionMs, playNext);

  const duration = audioState?.durationMs ?? 0;
  const progressPercent = duration > 0 ? (currentPositionMs / duration) * 100 : 0;

  const handleProgressBarClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!duration) return;
    const rect = e.currentTarget.getBoundingClientRect();
    const pos = (e.clientX - rect.left) / rect.width;
    handleSeek(pos * duration);
  };

  return (
    <div className="bg-bg-surface border-b border-border-subtle h-14 flex flex-col shrink-0 relative z-40">
      {/* Progress Bar (slim line at top) */}
      <div 
        className="h-1 bg-bg-input w-full cursor-pointer group absolute top-0 left-0"
        onClick={handleProgressBarClick}
      >
        <div 
          className="h-full bg-accent group-hover:bg-accent-hover transition-colors"
          style={{ width: `${progressPercent}%` }}
        />
      </div>

      <div className="flex-1 flex items-center justify-between px-4 mt-1">
        {/* Left: Info */}
        <div className="flex items-center gap-3 w-1/3 min-w-0">
          <SongInfo song={song ? { title: song.title, artist: song.artist } : null} />
        </div>

        {/* Center: Controls */}
        <div className="flex flex-col items-center justify-center gap-1 w-1/3">
          <PlaybackControls
            isPlaying={isPlaying}
            isLooping={isLooping}
            isShuffle={isShuffle}
            hasSong={!!song}
            onPlayPause={handlePlayPause}
            onPrevious={playPrevious}
            onNext={playNext}
            onToggleLoop={toggleLoop}
            onToggleShuffle={toggleShuffle}
          />
        </div>

        {/* Right: Tools & Time */}
        <div className="flex items-center justify-end gap-4 w-1/3">
          <div className="text-mono text-caption text-text-tertiary tabular-nums">
            {formatTime(currentPositionMs)} / {formatTime(duration)}
          </div>
          <ActiveTasksIndicator />
        </div>
      </div>
    </div>
  );
}
