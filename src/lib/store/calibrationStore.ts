import { create } from "zustand";
import type { Song, TimingPoint } from "@/lib/types";
import { getCalibration, saveTimingPoints } from "@/lib/calibration";

const PERSIST_DEBOUNCE_MS = 600;

interface CalibrationState {
  songId: string | null;
  durationMs: number;
  timingPoints: TimingPoint[];
  selectedTpIndex: number | null;
  isCalibrating: boolean;
  tapPositions: number[];
  detectedBpm: number | null;

  viewportStartMs: number;
  viewportEndMs: number;

  isDirty: boolean;
  isLoaded: boolean;

  loadForSong: (song: Song, durationMs: number) => Promise<void>;
  setTimingPoints: (tps: TimingPoint[]) => void;
  addTimingPoint: (tp: TimingPoint) => void;
  updateTimingPoint: (index: number, patch: Partial<TimingPoint>) => void;
  removeTimingPoint: (index: number) => void;
  clearAll: () => void;
  selectTimingPoint: (index: number | null) => void;

  setIsCalibrating: (calibrating: boolean) => void;
  pushTap: (positionMs: number) => void;
  resetTaps: () => void;
  setDetectedBpm: (bpm: number | null) => void;

  setViewport: (startMs: number, endMs: number) => void;
  zoomBy: (factor: number, centerMs?: number) => void;
  zoomToFit: () => void;
  panBy: (deltaMs: number) => void;

  persistNow: () => Promise<void>;
  reset: () => void;
}

let persistTimer: ReturnType<typeof setTimeout> | null = null;

function clampViewport(
  startMs: number,
  endMs: number,
  durationMs: number,
): { startMs: number; endMs: number } {
  if (durationMs <= 0) {
    return { startMs: 0, endMs: 0 };
  }
  const requestedSpan = endMs - startMs;
  const minSpan = Math.min(50, durationMs);

  let start = startMs;
  let end = endMs;

  if (start < 0) {
    start = 0;
    end = Math.min(start + requestedSpan, durationMs);
  }
  if (end > durationMs) {
    end = durationMs;
    start = Math.max(end - requestedSpan, 0);
  }
  if (start < 0) {
    start = 0;
    end = durationMs;
  }
  if (end - start < minSpan) {
    if (durationMs <= minSpan) {
      start = 0;
      end = durationMs;
    } else {
      end = Math.min(start + minSpan, durationMs);
      start = Math.max(end - minSpan, 0);
    }
  }

  return { startMs: start, endMs: end };
}

function sortTimingPoints(tps: TimingPoint[]): TimingPoint[] {
  return [...tps].sort((a, b) => a.offsetMs - b.offsetMs);
}

function schedulePersist(getState: () => CalibrationState) {
  if (persistTimer !== null) {
    clearTimeout(persistTimer);
  }
  persistTimer = setTimeout(() => {
    persistTimer = null;
    void persistInternal(getState);
  }, PERSIST_DEBOUNCE_MS);
}

function canPersist(state: CalibrationState): boolean {
  return state.songId !== null && state.isDirty && state.isLoaded;
}

async function persistInternal(getState: () => CalibrationState) {
  const state = getState();
  if (!canPersist(state)) return;
  try {
    await saveTimingPoints(state.songId!, state.timingPoints);
    useCalibrationStore.setState({ isDirty: false });
  } catch (err) {
    console.error("[calibrationStore] persist failed:", err);
  }
}

export const useCalibrationStore = create<CalibrationState>((set, get) => ({
  songId: null,
  durationMs: 0,
  timingPoints: [],
  selectedTpIndex: null,
  isCalibrating: false,
  tapPositions: [],
  detectedBpm: null,
  viewportStartMs: 0,
  viewportEndMs: 0,
  isDirty: false,
  isLoaded: false,

  loadForSong: async (song, durationMs) => {
    const safeDuration = durationMs > 0 ? durationMs : 0;
    try {
      const tps = await getCalibration(song.id);
      set({
        songId: song.id,
        durationMs: safeDuration,
        timingPoints: sortTimingPoints(tps),
        selectedTpIndex: null,
        isCalibrating: false,
        tapPositions: [],
        detectedBpm: null,
        viewportStartMs: 0,
        viewportEndMs: safeDuration,
        isDirty: false,
        isLoaded: true,
      });
    } catch (err) {
      console.error("[calibrationStore] load failed:", err);
      set({
        songId: song.id,
        durationMs: safeDuration,
        timingPoints: [],
        viewportStartMs: 0,
        viewportEndMs: safeDuration,
        isDirty: false,
        isLoaded: true,
      });
    }
  },

  setTimingPoints: (tps) => {
    set({ timingPoints: sortTimingPoints(tps), isDirty: true });
    schedulePersist(get);
  },

  addTimingPoint: (tp) => {
    const sorted = sortTimingPoints([...get().timingPoints, tp]);
    const newIndex = sorted.findIndex(
      (p) => p.offsetMs === tp.offsetMs && p.bpm === tp.bpm,
    );
    set({ timingPoints: sorted, selectedTpIndex: newIndex, isDirty: true });
    schedulePersist(get);
  },

  updateTimingPoint: (index, patch) => {
    const tps = get().timingPoints;
    if (index < 0 || index >= tps.length) return;
    const updated = tps.map((tp, i) => (i === index ? { ...tp, ...patch } : tp));
    const sorted = sortTimingPoints(updated);
    const moved = sorted.findIndex(
      (p) => p.offsetMs === updated[index].offsetMs && p.bpm === updated[index].bpm,
    );
    set({ timingPoints: sorted, selectedTpIndex: moved, isDirty: true });
    schedulePersist(get);
  },

  removeTimingPoint: (index) => {
    const tps = get().timingPoints;
    if (index < 0 || index >= tps.length) return;
    const next = tps.filter((_, i) => i !== index);
    const currentSel = get().selectedTpIndex;
    let newSel: number | null = currentSel;
    if (currentSel === index) newSel = null;
    else if (currentSel !== null && currentSel > index) newSel = currentSel - 1;
    set({ timingPoints: next, selectedTpIndex: newSel, isDirty: true });
    schedulePersist(get);
  },

  clearAll: () => {
    set({
      timingPoints: [],
      selectedTpIndex: null,
      isCalibrating: false,
      tapPositions: [],
      detectedBpm: null,
      isDirty: true,
    });
    schedulePersist(get);
  },

  selectTimingPoint: (index) => set({ selectedTpIndex: index }),

  setIsCalibrating: (calibrating) => {
    if (!calibrating) {
      set({ isCalibrating: false, tapPositions: [], detectedBpm: null });
    } else {
      set({ isCalibrating: true, tapPositions: [], detectedBpm: null });
    }
  },

  pushTap: (positionMs) => {
    set((s) => ({ tapPositions: [...s.tapPositions, positionMs] }));
  },

  resetTaps: () => set({ tapPositions: [], detectedBpm: null }),

  setDetectedBpm: (bpm) => set({ detectedBpm: bpm }),

  setViewport: (startMs, endMs) => {
    const { durationMs } = get();
    const clamped = clampViewport(startMs, endMs, durationMs);
    set({ viewportStartMs: clamped.startMs, viewportEndMs: clamped.endMs });
  },

  zoomBy: (factor, centerMs) => {
    const { viewportStartMs, viewportEndMs, durationMs } = get();
    const currentSpan = viewportEndMs - viewportStartMs;
    if (currentSpan <= 0) return;
    const center = centerMs ?? (viewportStartMs + viewportEndMs) / 2;
    const newSpan = Math.max(50, currentSpan / factor);
    let start = center - newSpan / 2;
    let end = center + newSpan / 2;
    const clamped = clampViewport(start, end, durationMs);
    set({ viewportStartMs: clamped.startMs, viewportEndMs: clamped.endMs });
  },

  zoomToFit: () => {
    const { durationMs } = get();
    set({ viewportStartMs: 0, viewportEndMs: durationMs });
  },

  panBy: (deltaMs) => {
    const { viewportStartMs, viewportEndMs, durationMs } = get();
    const span = viewportEndMs - viewportStartMs;
    let start = viewportStartMs + deltaMs;
    let end = viewportEndMs + deltaMs;
    if (start < 0) {
      start = 0;
      end = span;
    }
    if (end > durationMs) {
      end = durationMs;
      start = durationMs - span;
    }
    set({ viewportStartMs: start, viewportEndMs: end });
  },

  persistNow: async () => {
    if (persistTimer !== null) {
      clearTimeout(persistTimer);
      persistTimer = null;
    }
    await persistInternal(get);
  },

  reset: () => {
    if (persistTimer !== null) {
      clearTimeout(persistTimer);
      persistTimer = null;
    }
    set({
      songId: null,
      durationMs: 0,
      timingPoints: [],
      selectedTpIndex: null,
      isCalibrating: false,
      tapPositions: [],
      detectedBpm: null,
      viewportStartMs: 0,
      viewportEndMs: 0,
      isDirty: false,
      isLoaded: false,
    });
  },
}));
