import type { Song } from "@/lib/types";

interface SongRowProps {
  song: Song;
  index: number;
  isSelected: boolean;
  onSelect: (id: string) => void;
  onMissingClick: (song: Song) => void;
  onContextMenu: (e: React.MouseEvent, song: Song) => void;
}

export function SongRow({ song, index, isSelected, onSelect, onMissingClick, onContextMenu }: SongRowProps) {
  return (
    <div
      className={`grid grid-cols-[36px_minmax(0,2fr)_minmax(0,1.5fr)_minmax(0,3fr)_90px] h-9 cursor-pointer transition-colors duration-120 group border-b border-border-subtle ${
        isSelected
          ? "bg-accent-bg border-l-2 border-l-accent"
          : "border-l-2 border-l-transparent hover:bg-bg-surface"
      }`}
      onClick={() => onSelect(song.id)}
      onContextMenu={(e) => onContextMenu(e, song)}
    >
      <div className="px-2 flex items-center justify-center text-caption text-text-tertiary border-r border-border-subtle">
        {index + 1}
      </div>
      <div className="px-3 flex items-center text-body text-text-primary truncate border-r border-border-subtle">
        <span className="truncate">{song.title}</span>
      </div>
      <div className="px-3 flex items-center text-body text-text-secondary truncate border-r border-border-subtle">
        <span className="truncate">{song.artist ?? ""}</span>
      </div>
      <div className="px-3 flex items-center text-mono text-text-tertiary truncate border-r border-border-subtle">
        <span className="truncate">{song.audioPath}</span>
      </div>
      <div className="px-2 flex items-center justify-center">
        {song.audioMissing ? (
          <button
            onClick={(e) => {
              e.stopPropagation();
              onMissingClick(song);
            }}
            className="text-caption text-text-danger bg-danger-bg px-2 py-0.5 border border-danger-border rounded-sm hover:opacity-80 cursor-pointer transition-opacity"
          >
            Faltante
          </button>
        ) : (
          <span className="text-caption text-text-success flex items-center gap-1.5">
            <span className="inline-block w-1.5 h-1.5 rounded-full bg-text-success" />
            Listo
          </span>
        )}
      </div>
    </div>
  );
}
