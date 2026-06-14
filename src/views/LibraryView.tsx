import { useEffect, useState, useCallback, useMemo } from "react";
import { getLibraryWithStatus, deleteSong } from "@/lib/library";
import { useLibraryStore } from "@/lib/store";
import type { Song } from "@/lib/types";
import {
  AddSongDialog,
  EditSongDialog,
  LibraryToolbar,
  MissingFileDialog,
  SongList,
  StatusBar,
} from "@/components/Library";

interface ContextMenu {
  x: number;
  y: number;
  song: Song;
}

interface LibraryViewProps {
  onOpenSong?: (song: Song) => void;
}

export function LibraryView({ onOpenSong }: LibraryViewProps) {
  const songs = useLibraryStore((s) => s.songs);
  const selectedSongId = useLibraryStore((s) => s.selectedSongId);
  const setSelectedSongId = useLibraryStore((s) => s.setSelectedSongId);
  const setSongs = useLibraryStore((s) => s.setSongs);
  const setIsLoading = useLibraryStore((s) => s.setIsLoading);
  const removeSongFromStore = useLibraryStore((s) => s.removeSongFromStore);
  const searchQuery = useLibraryStore((s) => s.searchQuery);
  const sortField = useLibraryStore((s) => s.sortField);
  const sortDirection = useLibraryStore((s) => s.sortDirection);

  const filteredSongs = useMemo(() => {
    let list = songs;
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      list = songs.filter(
        (s) =>
          s.title.toLowerCase().includes(q) ||
          (s.artist && s.artist.toLowerCase().includes(q)),
      );
    }
    return [...list].sort((a, b) => {
      const aVal = (a[sortField] ?? "").toLowerCase();
      const bVal = (b[sortField] ?? "").toLowerCase();
      const cmp = aVal.localeCompare(bVal);
      return sortDirection === "asc" ? cmp : -cmp;
    });
  }, [songs, searchQuery, sortField, sortDirection]);

  const missingCount = useMemo(() => songs.filter((s) => s.audioMissing).length, [songs]);

  const [showAddDialog, setShowAddDialog] = useState(false);
  const [editSong, setEditSong] = useState<Song | null>(null);
  const [missingSong, setMissingSong] = useState<Song | null>(null);
  const [contextMenu, setContextMenu] = useState<ContextMenu | null>(null);

  useEffect(() => {
    setIsLoading(true);
    getLibraryWithStatus()
      .then((lib) => setSongs(lib.songs))
      .catch(console.error)
      .finally(() => setIsLoading(false));
  }, [setSongs, setIsLoading]);

  useEffect(() => {
    const CHECK_INTERVAL = 30_000;
    const interval = setInterval(() => {
      getLibraryWithStatus()
        .then((lib) => setSongs(lib.songs))
        .catch(() => {});
    }, CHECK_INTERVAL);

    const onFocus = () => {
      getLibraryWithStatus()
        .then((lib) => setSongs(lib.songs))
        .catch(() => {});
    };
    window.addEventListener("focus", onFocus);

    return () => {
      clearInterval(interval);
      window.removeEventListener("focus", onFocus);
    };
  }, [setSongs]);

  const selectedSong = songs.find((s) => s.id === selectedSongId) ?? null;

  const closeContextMenu = useCallback(() => setContextMenu(null), []);

  useEffect(() => {
    if (contextMenu) {
      window.addEventListener("click", closeContextMenu);
      return () => window.removeEventListener("click", closeContextMenu);
    }
  }, [contextMenu, closeContextMenu]);

  function handleDeselect() {
    setSelectedSongId(null);
  }

  function handleEditSong() {
    if (selectedSong) setEditSong(selectedSong);
  }

  async function handleDeleteSong() {
    if (!selectedSong) return;
    const confirmed = window.confirm(
      `¿Eliminar "${selectedSong.title}" de la biblioteca?\n\nEl archivo de audio no se modificará.`,
    );
    if (!confirmed) return;
    try {
      await deleteSong(selectedSong.id);
      removeSongFromStore(selectedSong.id);
    } catch {
      // song may already be gone
    }
  }

  function handleContextMenu(e: React.MouseEvent, song: Song) {
    e.preventDefault();
    setSelectedSongId(song.id);
    setContextMenu({ x: e.clientX, y: e.clientY, song });
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Delete" && selectedSong) {
      e.preventDefault();
      handleDeleteSong();
    }
    if (e.key === "Escape") {
      handleDeselect();
      setContextMenu(null);
    }
    if (e.key === "Enter" && selectedSong && onOpenSong) {
      e.preventDefault();
      onOpenSong(selectedSong);
    }
  }

  return (
    <div className="flex flex-col h-full" onKeyDown={handleKeyDown} tabIndex={0}>
      <LibraryToolbar
        selectedSong={selectedSong}
        onAddSong={() => setShowAddDialog(true)}
        onEditSong={handleEditSong}
      />
      <SongList
        songs={filteredSongs}
        selectedSongId={selectedSongId}
        onSelect={setSelectedSongId}
        onDoubleClick={(song) => onOpenSong?.(song)}
        onMissingClick={setMissingSong}
        onDeselect={handleDeselect}
        onContextMenu={handleContextMenu}
      />
      <StatusBar totalCount={songs.length} missingCount={missingCount} />
      {showAddDialog && <AddSongDialog onClose={() => setShowAddDialog(false)} />}
      {editSong && <EditSongDialog song={editSong} onClose={() => setEditSong(null)} />}
      {missingSong && (
        <MissingFileDialog song={missingSong} onClose={() => setMissingSong(null)} />
      )}
      {contextMenu && (
        <div
          className="fixed bg-bg-raised border border-border-subtle rounded-md shadow-modal z-50 py-1 min-w-[140px]"
          style={{ left: contextMenu.x, top: contextMenu.y }}
        >
          <button
            className="w-full text-left px-3 py-1.5 text-caption text-text-primary hover:bg-bg-surface cursor-pointer"
            onClick={() => {
              setEditSong(contextMenu.song);
              setContextMenu(null);
            }}
          >
            Editar
          </button>
          <button
            className="w-full text-left px-3 py-1.5 text-caption text-text-danger hover:bg-bg-surface cursor-pointer"
            onClick={() => {
              setSelectedSongId(contextMenu.song.id);
              setContextMenu(null);
              handleDeleteSong();
            }}
          >
            Eliminar
          </button>
          {contextMenu.song.audioMissing && (
            <button
              className="w-full text-left px-3 py-1.5 text-caption text-text-primary hover:bg-bg-surface cursor-pointer"
              onClick={() => {
                setMissingSong(contextMenu.song);
                setContextMenu(null);
              }}
            >
              Reasignar ruta
            </button>
          )}
        </div>
      )}
    </div>
  );
}
