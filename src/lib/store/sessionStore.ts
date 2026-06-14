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

  setCurrentSongId: (id: string | null) => void;
  setIsPlaying: (playing: boolean) => void;
  setPlaybackSpeed: (speed: number) => void;
  setCurrentPositionMs: (position: number) => void;
  setAudioState: (info: { durationMs: number; sampleRate: number; channels: number; peaks: number[] } | null) => void;
  updatePeaks: (peaks: number[]) => void;
  setDecodeProgress: (progress: number) => void;
  resetSession: () => void;
}

const INITIAL_STATE = {
  currentSongId: null,
  isPlaying: false,
  playbackSpeed: 1.0,
  currentPositionMs: 0,
  audioState: null,
  decodeProgress: 1.0,
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
  resetSession: () => set(INITIAL_STATE),
}));
