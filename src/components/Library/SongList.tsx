import type { Song } from "@/lib/types";
import { useLibraryStore } from "@/lib/store";
import { SongRow } from "./SongRow";

interface SongListProps {
  songs: Song[];
  selectedSongId: string | null;
  onSelect: (id: string) => void;
  onDoubleClick: (song: Song) => void;
  onMissingClick: (song: Song) => void;
  onDeselect: () => void;
  onContextMenu: (e: React.MouseEvent, song: Song) => void;
}

const columns = [
  { field: null, label: "#", className: "w-[36px]" },
  { field: "title" as const, label: "Título" },
  { field: "artist" as const, label: "Artista" },
  { field: "audioPath" as const, label: "Ruta del archivo" },
  { field: null, label: "Estado", className: "w-[90px]" },
];

export function SongList({ songs, selectedSongId, onSelect, onDoubleClick, onMissingClick, onDeselect, onContextMenu }: SongListProps) {
  const sortField = useLibraryStore((s) => s.sortField);
  const sortDirection = useLibraryStore((s) => s.sortDirection);
  const toggleSort = useLibraryStore((s) => s.toggleSort);

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <div className="grid grid-cols-[36px_minmax(0,2fr)_minmax(0,1.5fr)_minmax(0,3fr)_90px] bg-bg-surface border-b border-border-subtle shrink-0 border-l-2 border-l-transparent">
        {columns.map((col) => (
          <div
            key={col.label}
            className={`px-3 h-7 flex items-center text-caption text-text-secondary select-none border-r border-border-subtle last:border-r-0 ${
              col.field ? "cursor-pointer hover:text-text-primary transition-colors" : "justify-center"
            } ${col.className ?? ""}`}
            onClick={() => col.field && toggleSort(col.field)}
          >
            <span className="truncate flex items-center gap-1">
              {col.label}
              {col.field && sortField === col.field && (
                <span className="text-accent text-[10px]">
                  {sortDirection === "asc" ? "▲" : "▼"}
                </span>
              )}
            </span>
          </div>
        ))}
      </div>
      <div className="flex-1 overflow-y-auto" onClick={(e) => { if (e.target === e.currentTarget) onDeselect(); }}>
        {songs.length === 0 ? (
          <div className="flex-1 flex items-center justify-center h-full">
            <p className="text-body text-text-tertiary">
              Biblioteca vacía — haz clic en &quot;Agregar&quot; para comenzar
            </p>
          </div>
        ) : (
          songs.map((song, i) => (
            <SongRow
              key={song.id}
              song={song}
              index={i}
              isSelected={song.id === selectedSongId}
              onSelect={onSelect}
              onDoubleClick={onDoubleClick}
              onMissingClick={onMissingClick}
              onContextMenu={onContextMenu}
            />
          ))
        )}
      </div>
    </div>
  );
}
