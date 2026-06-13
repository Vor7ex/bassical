import { pickAudioFile } from "@/lib/dialog";
import { addSong, deleteSong } from "@/lib/library";
import { useLibraryStore } from "@/lib/store";
import type { Song } from "@/lib/types";
import { useState } from "react";

interface LibraryToolbarProps {
  selectedSong: Song | null;
  onAddSong: () => void;
  onEditSong: () => void;
}

export function LibraryToolbar({ selectedSong, onAddSong, onEditSong }: LibraryToolbarProps) {
  const searchQuery = useLibraryStore((s) => s.searchQuery);
  const setSearchQuery = useLibraryStore((s) => s.setSearchQuery);
  const removeSongFromStore = useLibraryStore((s) => s.removeSongFromStore);
  const addSongToStore = useLibraryStore((s) => s.addSongToStore);
  const [isDeleting, setIsDeleting] = useState(false);

  async function handleDelete() {
    if (!selectedSong) return;
    const confirmed = window.confirm(
      `¿Eliminar "${selectedSong.title}" de la biblioteca?\n\nEl archivo de audio no se modificará.`,
    );
    if (!confirmed) return;
    setIsDeleting(true);
    try {
      await deleteSong(selectedSong.id);
      removeSongFromStore(selectedSong.id);
    } catch {
      // song may already be gone
    } finally {
      setIsDeleting(false);
    }
  }

  async function handleQuickAdd() {
    const path = await pickAudioFile();
    if (!path) return;
    const filename = path.split(/[/\\]/).pop() ?? "";
    const nameWithoutExt = filename.replace(/\.[^.]+$/, "");
    try {
      const song = await addSong(nameWithoutExt, undefined, path);
      addSongToStore(song);
    } catch {
      onAddSong();
    }
  }

  return (
    <div className="bg-bg-surface border-b border-border-subtle px-3 h-10 flex items-center gap-2 shrink-0">
      <button
        onClick={handleQuickAdd}
        className="bg-accent text-bg-root px-3 h-7 text-caption font-medium rounded-sm hover:bg-accent-hover cursor-pointer transition-colors"
      >
        + Agregar
      </button>
      <div className="w-px h-5 bg-border-subtle" />
      <button
        onClick={handleDelete}
        disabled={!selectedSong || isDeleting}
        className="bg-bg-input text-text-secondary px-3 h-7 text-caption rounded-sm border border-border-subtle hover:text-text-primary hover:border-border-strong cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed transition-all"
      >
        Eliminar
      </button>
      <button
        onClick={onEditSong}
        disabled={!selectedSong}
        className="bg-bg-input text-text-secondary px-3 h-7 text-caption rounded-sm border border-border-subtle hover:text-text-primary hover:border-border-strong cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed transition-all"
      >
        Editar
      </button>
      <div className="flex-1" />
      <input
        type="text"
        value={searchQuery}
        onChange={(e) => setSearchQuery(e.target.value)}
        placeholder="Buscar..."
        className="bg-bg-input border border-border-subtle text-text-primary text-caption px-3 h-7 w-48 rounded-sm placeholder:text-text-tertiary focus:outline-none focus:border-accent transition-colors"
      />
    </div>
  );
}
