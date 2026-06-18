import type { Song } from "@/lib/types";

interface SongRowProps {
  song: Song;
  index: number;
  isSelected: boolean;
  onSelect: (id: string) => void;
  onDoubleClick: (song: Song) => void;
  onContextMenu: (e: React.MouseEvent, song: Song) => void;
}

export function SongRow({ song, index, isSelected, onSelect, onDoubleClick, onContextMenu }: SongRowProps) {
  return (
    <div
      className={`grid grid-cols-[36px_minmax(0,2fr)_minmax(0,1.5fr)_minmax(0,1.5fr)_80px] h-9 cursor-pointer transition-colors duration-120 group border-b border-border-subtle ${
        isSelected
          ? "bg-accent-bg border-l-2 border-l-accent"
          : "border-l-2 border-l-transparent hover:bg-bg-surface"
      } ${song.audioMissing ? "text-text-danger bg-danger-bg/10 hover:bg-danger-bg/20" : ""}`}
      onClick={() => onSelect(song.id)}
      onDoubleClick={() => onDoubleClick(song)}
      onContextMenu={(e) => onContextMenu(e, song)}
    >
      <div className="px-2 flex items-center justify-center text-caption text-text-tertiary border-r border-border-subtle">
        {index + 1}
      </div>
      <div className="px-3 flex items-center text-body text-text-primary truncate border-r border-border-subtle">
        <span className="truncate">{song.title}</span>
      </div>
      <div className={`px-3 flex items-center text-body truncate border-r border-border-subtle ${song.audioMissing ? "text-text-danger" : "text-text-secondary"}`}>
        <span className="truncate">{song.artist ?? ""}</span>
      </div>
      <div className={`px-3 flex items-center text-body truncate border-r border-border-subtle ${song.audioMissing ? "text-text-danger" : "text-text-secondary"}`}>
        <span className="truncate">{song.album ?? ""}</span>
      </div>
      <div className={`px-2 flex items-center justify-center text-mono text-caption ${song.audioMissing ? "text-text-danger font-medium" : "text-text-tertiary"}`}>
        {song.audioMissing ? "Error" : song.bpm ? Math.round(song.bpm) : "--"}
      </div>
    </div>
  );
}
