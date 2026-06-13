import { create } from "zustand";

interface TabState {
  // Se expandirá en Sprint 4-5
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  measures: any[];
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  setMeasures: (measures: any[]) => void;
}

export const useTabStore = create<TabState>((set) => ({
  measures: [],
  setMeasures: (measures) => set({ measures }),
}));
