import { create } from "zustand";

interface AudioState {
  durationMs: number;
  sampleRate: number;
  channels: number;
  peaks: number[];
}

interface SessionState {
  currentSongId: string | null;
  isPlaying: boolean;
  playbackSpeed: number;
  currentPositionMs: number;
  audioState: AudioState | null;
  decodeProgress: number;

  queue: string[];
  isLooping: boolean;
  isShuffle: boolean;

  setCurrentSongId: (id: string | null) => void;
  setIsPlaying: (playing: boolean) => void;
  setPlaybackSpeed: (speed: number) => void;
  setCurrentPositionMs: (position: number) => void;
  setAudioState: (info: { durationMs: number; sampleRate: number; channels: number; peaks: number[] } | null) => void;
  updatePeaks: (peaks: number[]) => void;
  setDecodeProgress: (progress: number) => void;
  setQueue: (queue: string[]) => void;
  toggleLoop: () => void;
  toggleShuffle: () => void;
  playNext: () => void;
  playPrevious: () => void;
  resetSession: () => void;
}

const INITIAL_STATE = {
  currentSongId: null,
  isPlaying: false,
  playbackSpeed: 1.0,
  currentPositionMs: 0,
  audioState: null,
  decodeProgress: 1.0,
  queue: [],
  isLooping: false,
  isShuffle: false,
};

export const useSessionStore = create<SessionState>((set) => ({
  ...INITIAL_STATE,

  setCurrentSongId: (id) => set({ currentSongId: id }),
  setIsPlaying: (playing) => set({ isPlaying: playing }),
  setPlaybackSpeed: (speed) => set({ playbackSpeed: speed }),
  setCurrentPositionMs: (position) => set({ currentPositionMs: position }),
  setAudioState: (info) =>
    set({
      audioState: info
        ? {
            durationMs: info.durationMs,
            sampleRate: info.sampleRate,
            channels: info.channels,
            peaks: info.peaks,
          }
        : null,
    }),
  updatePeaks: (peaks) =>
    set((state) => ({
      audioState: state.audioState
        ? { ...state.audioState, peaks }
        : null,
    })),
  setDecodeProgress: (progress) => set({ decodeProgress: progress }),
  setQueue: (queue) => set({ queue }),
  toggleLoop: () => set((s) => ({ isLooping: !s.isLooping })),
  toggleShuffle: () => set((s) => ({ isShuffle: !s.isShuffle })),
  playNext: () =>
    set((s) => {
      if (s.queue.length === 0) return {};
      if (s.isShuffle) {
        const nextId = s.queue[Math.floor(Math.random() * s.queue.length)];
        return { currentSongId: nextId };
      }
      const idx = s.queue.indexOf(s.currentSongId ?? "");
      if (idx === -1 || idx === s.queue.length - 1) {
        return s.isLooping ? { currentSongId: s.queue[0] } : {};
      }
      return { currentSongId: s.queue[idx + 1] };
    }),
  playPrevious: () =>
    set((s) => {
      if (s.queue.length === 0) return {};
      if (s.isShuffle) {
        const nextId = s.queue[Math.floor(Math.random() * s.queue.length)];
        return { currentSongId: nextId };
      }
      const idx = s.queue.indexOf(s.currentSongId ?? "");
      if (idx <= 0) {
        return s.isLooping ? { currentSongId: s.queue[s.queue.length - 1] } : {};
      }
      return { currentSongId: s.queue[idx - 1] };
    }),
  resetSession: () => set(INITIAL_STATE),
}));
