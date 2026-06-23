import { create } from "zustand";
import { toggleMetronome, getMetronomeState } from "@/lib/metronome";

interface MetronomeStore {
  enabled: boolean;
  setEnabled: (v: boolean) => void;
  toggle: () => Promise<void>;
  sync: () => Promise<void>;
}

export const useMetronomeStore = create<MetronomeStore>((set) => ({
  enabled: false,

  setEnabled: (v) => set({ enabled: v }),

  toggle: async () => {
    const newState = await toggleMetronome();
    set({ enabled: newState });
  },

  sync: async () => {
    const enabled = await getMetronomeState();
    set({ enabled });
  },
}));
