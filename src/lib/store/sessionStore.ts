import { create } from "zustand";

interface SessionState {
  currentSongId: string | null;
  isPlaying: boolean;
  playbackSpeed: number;
  setCurrentSongId: (id: string | null) => void;
  setIsPlaying: (playing: boolean) => void;
  setPlaybackSpeed: (speed: number) => void;
}

export const useSessionStore = create<SessionState>((set) => ({
  currentSongId: null,
  isPlaying: false,
  playbackSpeed: 1.0,
  setCurrentSongId: (id) => set({ currentSongId: id }),
  setIsPlaying: (playing) => set({ isPlaying: playing }),
  setPlaybackSpeed: (speed) => set({ playbackSpeed: speed }),
}));
