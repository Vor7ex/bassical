import { create } from "zustand";
import type { Song } from "@/lib/types";

export type SortField = "title" | "artist" | "audioPath";
export type SortDirection = "asc" | "desc";

interface LibraryState {
  songs: Song[];
  searchQuery: string;
  selectedSongId: string | null;
  isLoading: boolean;
  sortField: SortField;
  sortDirection: SortDirection;

  setSongs: (songs: Song[]) => void;
  setSearchQuery: (query: string) => void;
  setSelectedSongId: (id: string | null) => void;
  setIsLoading: (loading: boolean) => void;
  setSortField: (field: SortField) => void;
  toggleSort: (field: SortField) => void;
  addSongToStore: (song: Song) => void;
  removeSongFromStore: (id: string) => void;
  updateSongInStore: (song: Song) => void;
  getFilteredSongs: () => Song[];
  getMissingCount: () => number;
}

export const useLibraryStore = create<LibraryState>((set, get) => ({
  songs: [],
  searchQuery: "",
  selectedSongId: null,
  isLoading: false,
  sortField: "title",
  sortDirection: "asc",

  setSongs: (songs) => set({ songs }),
  setSearchQuery: (searchQuery) => set({ searchQuery }),
  setSelectedSongId: (selectedSongId) => set({ selectedSongId }),
  setIsLoading: (isLoading) => set({ isLoading }),
  setSortField: (sortField) => set({ sortField }),

  toggleSort: (field) =>
    set((state) => ({
      sortField: field,
      sortDirection: state.sortField === field && state.sortDirection === "asc" ? "desc" : "asc",
    })),

  addSongToStore: (song) => set((state) => ({ songs: [...state.songs, song] })),

  removeSongFromStore: (id) =>
    set((state) => ({
      songs: state.songs.filter((s) => s.id !== id),
      selectedSongId: state.selectedSongId === id ? null : state.selectedSongId,
    })),

  updateSongInStore: (song) =>
    set((state) => ({
      songs: state.songs.map((s) => (s.id === song.id ? song : s)),
    })),

  getFilteredSongs: () => {
    const { songs, searchQuery, sortField, sortDirection } = get();
    let filtered = songs;

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      filtered = songs.filter(
        (s) =>
          s.title.toLowerCase().includes(q) ||
          (s.artist && s.artist.toLowerCase().includes(q)),
      );
    }

    return [...filtered].sort((a, b) => {
      const aVal = (a[sortField] ?? "").toLowerCase();
      const bVal = (b[sortField] ?? "").toLowerCase();
      const cmp = aVal.localeCompare(bVal);
      return sortDirection === "asc" ? cmp : -cmp;
    });
  },

  getMissingCount: () => {
    return get().songs.filter((s) => s.audioMissing).length;
  },
}));
