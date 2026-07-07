import { create } from "zustand";
import {
  toggleMetronome,
  getMetronomeState,
  getMetronomeBalance,
  setMetronomeBalance,
} from "@/lib/metronome";

interface MetronomeStore {
  enabled: boolean;
  balance: number;
  setEnabled: (v: boolean) => void;
  toggle: () => Promise<void>;
  setBalance: (v: number) => Promise<void>;
  sync: () => Promise<void>;
}

export const useMetronomeStore = create<MetronomeStore>((set) => ({
  enabled: false,
  balance: 0.7,

  setEnabled: (v) => set({ enabled: v }),

  toggle: async () => {
    const newState = await toggleMetronome();
    set({ enabled: newState });
  },

  setBalance: async (v: number) => {
    const clamped = Math.max(0, Math.min(1, v));
    set({ balance: clamped });
    await setMetronomeBalance(clamped);
  },

  sync: async () => {
    const [enabled, balance] = await Promise.all([
      getMetronomeState(),
      getMetronomeBalance(),
    ]);
    set({ enabled, balance });
  },
}));