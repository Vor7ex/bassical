import { useEffect, useRef, useCallback } from "react";
import type { Song } from "@/lib/types";
import { usePracticePlayback } from "@/lib/usePracticePlayback";
import { useViewportPeaks } from "@/lib/useViewportPeaks";
import { computeBeatGrid } from "@/lib/beatGrid";
import { useCalibrationStore, useMetronomeStore } from "@/lib/store";
import { useTapCalibration } from "@/lib/useTapCalibration";
import { setMetronomeGrid } from "@/lib/metronome";
import { WaveformView, PlaybackControls, TimingPointMarker, TimingPointPanel } from "@/components/Audio";

interface AudioViewProps {
  song: Song;
  onBack: () => void;
}

interface AudioState {
  durationMs: number;
  sampleRate: number;
  channels: number;
  peaks: number[];
}

const SEEK_STEP_MS = 5000;
const MAX_ZOOM_FACTOR = 200;
const LOG_MAX_ZOOM = Math.log10(MAX_ZOOM_FACTOR);

function formatTime(ms: number): string {
  const totalSec = ms / 1000;
  const min = Math.floor(totalSec / 60);
  const sec = Math.floor(totalSec % 60);
  const tenths = Math.floor((ms % 1000) / 100);
  return `${min}:${String(sec).padStart(2, "0")}.${tenths}`;
}

function sliderToSpan(slider: number, durationMs: number): number {
  const factor = Math.pow(10, (slider / 100) * LOG_MAX_ZOOM);
  return Math.max(50, durationMs / factor);
}

function spanToSlider(span: number, durationMs: number): number {
  if (durationMs <= 0 || span >= durationMs) return 0;
  const factor = durationMs / span;
  return Math.min(100, (Math.log10(factor) / LOG_MAX_ZOOM) * 100);
}

const TEXT_INPUT_TYPES = new Set([
  "text", "number", "password", "email", "search",
  "tel", "url", "date", "datetime-local", "time",
]);

function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  const tag = target.tagName;
  if (tag === "TEXTAREA" || tag === "SELECT") return true;
  if (tag === "INPUT") {
    return TEXT_INPUT_TYPES.has((target as HTMLInputElement).type);
  }
  return false;
}

interface PlaybackKeyboardParams {
  isPlaying: boolean;
  isCalibrating: boolean;
  currentPositionMs: number;
  durationMs: number;
  onPlayPause: () => void;
  onSeek: (ms: number) => void;
  onTap: () => void;
  onToggleMetronome: () => void;
}

function usePlaybackKeyboard(params: PlaybackKeyboardParams) {
  const stateRef = useRef(params);
  stateRef.current = params;

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (isTypingTarget(e.target)) return;
      const s = stateRef.current;
      switch (e.code) {
        case "KeyM":
          e.preventDefault();
          s.onToggleMetronome();
          break;
        case "KeyT":
          if (!s.isCalibrating) return;
          e.preventDefault();
          s.onTap();
          break;
        case "Space":
          e.preventDefault();
          s.onPlayPause();
          break;
        case "ArrowLeft":
          e.preventDefault();
          s.onSeek(Math.max(0, s.currentPositionMs - SEEK_STEP_MS));
          break;
        case "ArrowRight":
          e.preventDefault();
          s.onSeek(Math.min(s.durationMs, s.currentPositionMs + SEEK_STEP_MS));
          break;
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);
}

interface ZoomToolbarProps {
  viewportStartMs: number;
  viewportEndMs: number;
  durationMs: number;
  onSliderChange: (span: number) => void;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onFit: () => void;
}

function ZoomToolbar({
  viewportStartMs,
  viewportEndMs,
  durationMs,
  onSliderChange,
  onZoomIn,
  onZoomOut,
  onFit,
}: ZoomToolbarProps) {
  const span = viewportEndMs - viewportStartMs;
  const sliderValue = spanToSlider(span, durationMs);
  const zoomLevel = span > 0 ? Math.round((durationMs / span) * 100) : 100;

  const handleSlider = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const newSpan = sliderToSpan(parseFloat(e.target.value), durationMs);
      onSliderChange(newSpan);
    },
    [durationMs, onSliderChange],
  );

  return (
    <div className="bg-bg-surface border-t border-border-subtle h-8 flex items-center justify-between px-3 shrink-0">
      <span className="text-mono text-caption text-text-tertiary">
        {formatTime(viewportStartMs)} – {formatTime(viewportEndMs)}
      </span>
      <div className="flex items-center gap-2">
        <button
          onClick={onZoomOut}
          className="text-text-tertiary hover:text-text-primary w-5 h-5 flex items-center justify-center text-body cursor-pointer transition-colors"
          aria-label="Alejar zoom"
        >
          −
        </button>
        <input
          type="range"
          className="zoom-slider w-32"
          min={0}
          max={100}
          step={0.5}
          value={sliderValue}
          onChange={handleSlider}
          aria-label="Nivel de zoom"
        />
        <button
          onClick={onZoomIn}
          className="text-text-tertiary hover:text-text-primary w-5 h-5 flex items-center justify-center text-body cursor-pointer transition-colors"
          aria-label="Acercar zoom"
        >
          +
        </button>
        <span className="text-mono text-caption text-text-secondary min-w-[48px] text-center">
          {zoomLevel >= 10000
            ? `${Math.round(zoomLevel / 1000)}K%`
            : `${zoomLevel}%`}
        </span>
        <div className="w-px h-4 bg-border-subtle mx-1" />
        <button
          onClick={onFit}
          className="text-caption text-text-tertiary hover:text-text-primary px-1.5 cursor-pointer transition-colors"
          aria-label="Ajustar a pantalla"
        >
          Fit
        </button>
      </div>
    </div>
  );
}

interface CalibrationPanelProps {
  detectedBpm: number | null;
  tapCount: number;
  acceptedCount: number;
  rejectedCount: number;
  canCommit: boolean;
  onTap: () => void;
  onCommit: () => void;
  onCancel: () => void;
  onBeginCalibrating: (index: number) => void;
  calibratingTpIndex: number | null;
}

interface WaveformAreaProps {
  audioState: AudioState;
  currentPositionMs: number;
  onSeek: (positionMs: number) => void;
  calibrationProps: CalibrationPanelProps;
}

function WaveformArea({
  audioState,
  currentPositionMs,
  onSeek,
  calibrationProps,
}: WaveformAreaProps) {
  const viewportStartMs = useCalibrationStore((s) => s.viewportStartMs);
  const viewportEndMs = useCalibrationStore((s) => s.viewportEndMs);
  const zoomBy = useCalibrationStore((s) => s.zoomBy);
  const panBy = useCalibrationStore((s) => s.panBy);
  const zoomToFit = useCalibrationStore((s) => s.zoomToFit);
  const setViewport = useCalibrationStore((s) => s.setViewport);
  const timingPoints = useCalibrationStore((s) => s.timingPoints);
  const selectedTpIndex = useCalibrationStore((s) => s.selectedTpIndex);
  const addTimingPoint = useCalibrationStore((s) => s.addTimingPoint);
  const updateTimingPoint = useCalibrationStore((s) => s.updateTimingPoint);
  const removeTimingPoint = useCalibrationStore((s) => s.removeTimingPoint);
  const selectTimingPoint = useCalibrationStore((s) => s.selectTimingPoint);

  const peaks = useViewportPeaks({
    overviewPeaks: audioState.peaks,
    viewportStartMs,
    viewportEndMs,
    durationMs: audioState.durationMs,
  });

  const beatGrid = computeBeatGrid(
    timingPoints,
    viewportStartMs,
    viewportEndMs,
    audioState.durationMs,
  );

  const handleSliderChange = useCallback(
    (newSpan: number) => {
      const center = (viewportStartMs + viewportEndMs) / 2;
      setViewport(center - newSpan / 2, center + newSpan / 2);
    },
    [viewportStartMs, viewportEndMs, setViewport],
  );

  const handleAddTp = useCallback(
    (offsetMs: number) => {
      addTimingPoint({ offsetMs, bpm: 120 });
    },
    [addTimingPoint],
  );

  return (
    <div className="flex-1 flex flex-row min-h-0 gap-0">
      <div className="flex-1 flex flex-col min-h-0 rounded-sm overflow-hidden border border-border-subtle">
        <WaveformView
          peaks={peaks}
          currentPositionMs={currentPositionMs}
          viewportStartMs={viewportStartMs}
          viewportEndMs={viewportEndMs}
          durationMs={audioState.durationMs}
          onSeek={onSeek}
          onZoom={zoomBy}
          onPan={panBy}
          beatGrid={beatGrid}
        >
          {timingPoints.map((tp, idx) => (
            <TimingPointMarker
              key={`${idx}-${tp.offsetMs}`}
              tp={tp}
              index={idx}
              isSelected={selectedTpIndex === idx}
              viewportStartMs={viewportStartMs}
              viewportEndMs={viewportEndMs}
              onSelect={selectTimingPoint}
            />
          ))}
        </WaveformView>
        <ZoomToolbar
          viewportStartMs={viewportStartMs}
          viewportEndMs={viewportEndMs}
          durationMs={audioState.durationMs}
          onSliderChange={handleSliderChange}
          onZoomIn={() => zoomBy(1.5)}
          onZoomOut={() => zoomBy(1 / 1.5)}
          onFit={zoomToFit}
        />
      </div>
      <TimingPointPanel
        timingPoints={timingPoints}
        selectedIndex={selectedTpIndex}
        durationMs={audioState.durationMs}
        defaultOffsetMs={currentPositionMs}
        onAdd={handleAddTp}
        onUpdate={updateTimingPoint}
        onRemove={removeTimingPoint}
        onSelect={selectTimingPoint}
        calibrationProps={calibrationProps}
      />
    </div>
  );
}

interface AudioMainContentProps {
  audioState: AudioState | null;
  currentPositionMs: number;
  onSeek: (positionMs: number) => void;
  calibrationProps: CalibrationPanelProps;
}

function AudioMainContent({
  audioState,
  currentPositionMs,
  onSeek,
  calibrationProps,
}: AudioMainContentProps) {
  if (!audioState) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <span className="text-body text-text-tertiary">Cargando audio...</span>
      </div>
    );
  }
  if (audioState.peaks.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <span className="text-body text-text-tertiary">
          Decodificando audio...
        </span>
      </div>
    );
  }
  return (
    <WaveformArea
      audioState={audioState}
      currentPositionMs={currentPositionMs}
      onSeek={onSeek}
      calibrationProps={calibrationProps}
    />
  );
}

interface StatusBarProps {
  playbackSpeed: number;
  decodeProgress: number;
  timingPointCount: number;
}

function StatusBar({
  playbackSpeed,
  decodeProgress,
  timingPointCount,
}: StatusBarProps) {
  return (
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
            Timing points: {timingPointCount}
          </span>
        )}
      </div>
      <span className="text-caption text-text-tertiary">Bassical</span>
    </div>
  );
}

function useCalibrationLifecycle(song: Song, audioState: AudioState | null) {
  const loadForSong = useCalibrationStore((s) => s.loadForSong);
  const persistNow = useCalibrationStore((s) => s.persistNow);
  const resetCalibration = useCalibrationStore((s) => s.reset);
  const loadedSongId = useCalibrationStore((s) => s.songId);

  const durationReady = !!audioState && audioState.durationMs > 0;
  const shouldLoad = durationReady && loadedSongId !== song.id;

  useEffect(() => {
    if (shouldLoad && audioState) {
      void loadForSong(song, audioState.durationMs);
    }
  }, [shouldLoad, audioState, song, loadForSong]);

  useEffect(() => {
    return () => {
      void persistNow().finally(() => resetCalibration());
    };
  }, [persistNow, resetCalibration]);
}

function useAutoScrollViewport(
  isPlaying: boolean,
  currentPositionMs: number,
  hasAudio: boolean,
) {
  const setViewport = useCalibrationStore((s) => s.setViewport);
  const viewportStartMs = useCalibrationStore((s) => s.viewportStartMs);
  const viewportEndMs = useCalibrationStore((s) => s.viewportEndMs);

  const viewportRef = useRef({ start: 0, end: 0 });
  viewportRef.current = { start: viewportStartMs, end: viewportEndMs };

  useEffect(() => {
    if (!isPlaying || !hasAudio) return;
    const { start, end } = viewportRef.current;
    const span = end - start;
    if (span <= 0) return;
    if (currentPositionMs > end) {
      setViewport(currentPositionMs - span * 0.1, currentPositionMs + span * 0.9);
    } else if (currentPositionMs < start) {
      setViewport(currentPositionMs - span * 0.9, currentPositionMs + span * 0.1);
    }
  }, [currentPositionMs, isPlaying, hasAudio, setViewport]);
}

function useMetronomeSync(
  songId: string | null,
  timingPoints: { offsetMs: number; bpm: number; timeSignature?: { numerator: number; denominator: number } }[],
  durationMs: number,
) {
  const syncMetronome = useMetronomeStore((s) => s.sync);

  useEffect(() => {
    if (!songId || durationMs <= 0) return;
    setMetronomeGrid(timingPoints, durationMs).catch(console.error);
  }, [songId, timingPoints, durationMs]);

  useEffect(() => {
    syncMetronome().catch(console.error);
  }, [syncMetronome]);
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

  const timingPoints = useCalibrationStore((s) => s.timingPoints);
  const songId = useCalibrationStore((s) => s.songId);

  const metronomeOn = useMetronomeStore((s) => s.enabled);
  const toggleMetronome = useMetronomeStore((s) => s.toggle);
  const balance = useMetronomeStore((s) => s.balance);
  const setBalance = useMetronomeStore((s) => s.setBalance);

  const handleBalanceChange = useCallback(
    (v: number) => {
      setBalance(v).catch(console.error);
    },
    [setBalance],
  );

  const {
    calibratingTpIndex,
    detectedBpm,
    tapCount,
    acceptedCount,
    rejectedCount,
    canCommit,
    beginCalibrating,
    handleFirstTap,
    commitTapPoint,
    cancelCalibrating,
  } = useTapCalibration(handleSeek, handlePlayPause, isPlaying);

  useCalibrationLifecycle(song, audioState);
  useAutoScrollViewport(isPlaying, currentPositionMs, !!audioState);
  useMetronomeSync(songId, timingPoints, audioState?.durationMs ?? 0);

  usePlaybackKeyboard({
    isPlaying,
    isCalibrating: calibratingTpIndex !== null,
    currentPositionMs,
    durationMs: audioState?.durationMs ?? 0,
    onPlayPause: handlePlayPause,
    onSeek: handleSeek,
    onTap: handleFirstTap,
    onToggleMetronome: toggleMetronome,
  });

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
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-1.5" title="Balance canción / metrónomo">
            <span className="text-caption text-text-tertiary" aria-hidden="true">
              ♪
            </span>
            <input
              type="range"
              className="balance-slider w-24"
              min={0}
              max={100}
              value={Math.round(balance * 100)}
              onChange={(e) =>
                handleBalanceChange(parseInt(e.target.value, 10) / 100)
              }
              aria-label="Balance canción / metrónomo"
            />
            <span className="text-caption text-text-tertiary" aria-hidden="true">
              ⏱
            </span>
            <span className="text-mono text-text-tertiary w-12 text-center">
              {Math.round(balance * 100)}/{Math.round((1 - balance) * 100)}
            </span>
          </div>
          <span className="text-mono text-text-tertiary text-caption">
          {audioState
            ? `${Math.floor(audioState.durationMs / 60000)}:${String(
                Math.floor((audioState.durationMs % 60000) / 1000),
              ).padStart(2, "0")}`
            : "--:--"}
        </span>
        </div>
      </div>

      <div className="flex-1 flex flex-col overflow-hidden p-4 gap-4">
        <AudioMainContent
          audioState={audioState}
          currentPositionMs={currentPositionMs}
          onSeek={handleSeek}
          calibrationProps={{
            detectedBpm,
            tapCount,
            acceptedCount,
            rejectedCount,
            canCommit,
            onTap: handleFirstTap,
            onCommit: commitTapPoint,
            onCancel: cancelCalibrating,
            onBeginCalibrating: beginCalibrating,
            calibratingTpIndex,
          }}
        />
      </div>

      <PlaybackControls
        isPlaying={isPlaying}
        currentPositionMs={currentPositionMs}
        durationMs={audioState?.durationMs ?? 0}
        playbackSpeed={playbackSpeed}
        speedDisabled={!fullBufferReady}
        metronomeOn={metronomeOn}
        onPlayPause={handlePlayPause}
        onSeek={handleSeek}
        onSpeedChange={handleSpeedChange}
        onToggleMetronome={toggleMetronome}
      />

      <StatusBar
        playbackSpeed={playbackSpeed}
        decodeProgress={decodeProgress}
        timingPointCount={timingPoints.length}
      />
    </div>
  );
}
