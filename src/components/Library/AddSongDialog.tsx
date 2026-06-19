import { useState } from "react";
import { pickAudioFile } from "@/lib/dialog";
import { addSong } from "@/lib/library";
import { extractMetadata } from "@/lib/audio";
import { useLibraryStore } from "@/lib/store";

interface AddSongDialogProps {
  onClose: () => void;
}

export function AddSongDialog({ onClose }: AddSongDialogProps) {
  const [title, setTitle] = useState("");
  const [artist, setArtist] = useState("");
  const [album, setAlbum] = useState("");
  const [genre, setGenre] = useState("");
  const [year, setYear] = useState("");
  const [audioPath, setAudioPath] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isAdding, setIsAdding] = useState(false);
  const addSongToStore = useLibraryStore((s) => s.addSongToStore);

  async function handlePickFile() {
    const path = await pickAudioFile();
    if (!path) return;
    setAudioPath(path);
    setError(null);
    const filename = path.split(/[/\\]/).pop() ?? "";
    setTitle(filename.replace(/\.[^.]+$/, ""));

    try {
      console.log("[extractMetadata] Llamando con path:", path);
      const metadata = await extractMetadata(path);
      console.log("[extractMetadata] Resultado:", JSON.stringify(metadata, null, 2));
      if (metadata.title) {
        setTitle(metadata.title);
      }
      if (metadata.artist) {
        setArtist(metadata.artist);
      }
      if (metadata.album) {
        setAlbum(metadata.album);
      }
      if (metadata.genre) {
        setGenre(metadata.genre);
      }
      if (metadata.year) {
        setYear(metadata.year);
      }
    } catch (e) {
      console.error("[extractMetadata] Error:", e);
    }
  }

  async function handleAdd() {
    if (!audioPath) return;
    if (!title.trim()) {
      setError("El título es obligatorio");
      return;
    }
    setIsAdding(true);
    setError(null);
    try {
      const yearNum = year.trim() ? parseInt(year, 10) : undefined;
      const song = await addSong(
        title.trim(),
        artist.trim() || undefined,
        audioPath,
        album.trim() || undefined,
        genre.trim() || undefined,
        yearNum,
      );
      addSongToStore(song);
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setIsAdding(false);
    }
  }

  return (
    <div
      className="fixed inset-0 bg-black/60 flex items-center justify-center z-40"
      onMouseDown={onClose}
    >
      <div
        className="bg-bg-raised border border-border-subtle w-[440px] rounded-md shadow-modal"
        onClick={(e) => e.stopPropagation()}
        onMouseDown={(e) => e.stopPropagation()}
        onKeyDown={(e) => {
          e.stopPropagation();
          if (e.key === "Escape") {
            onClose();
          } else if (e.key === "Enter") {
            const canSubmit = audioPath && title.trim() && !isAdding;
            if (canSubmit) handleAdd();
          }
        }}
      >
        <div className="bg-bg-surface px-4 h-9 flex items-center border-b border-border-subtle rounded-t-md">
          <span className="text-caption text-text-primary">Agregar canción</span>
        </div>
        <div className="p-4 space-y-3">
          <div>
            <label className="block text-caption text-text-secondary mb-1.5">
              Archivo de audio
            </label>
            <div className="flex gap-2">
              <input
                type="text"
                readOnly
                value={audioPath ?? ""}
                placeholder="Seleccionar archivo..."
                className="flex-1 bg-bg-input border border-border-subtle text-text-primary text-mono text-xs px-3 h-8 rounded-sm placeholder:text-text-tertiary"
              />
              <button
                onClick={handlePickFile}
                className="bg-bg-input text-text-secondary px-3 h-8 text-caption rounded-sm border border-border-subtle hover:text-text-primary hover:border-border-strong cursor-pointer transition-all"
              >
                Examinar
              </button>
            </div>
          </div>
          <div>
            <label className="block text-caption text-text-secondary mb-1.5">Título</label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              className="w-full bg-bg-input border border-border-subtle text-text-primary text-body px-3 h-8 rounded-sm focus:outline-none focus:border-accent transition-colors"
              autoFocus
            />
          </div>
          <div>
            <label className="block text-caption text-text-secondary mb-1.5">Artista</label>
            <input
              type="text"
              value={artist}
              onChange={(e) => setArtist(e.target.value)}
              placeholder="Opcional"
              className="w-full bg-bg-input border border-border-subtle text-text-primary text-body px-3 h-8 rounded-sm placeholder:text-text-tertiary focus:outline-none focus:border-accent transition-colors"
            />
          </div>
          <div>
            <label className="block text-caption text-text-secondary mb-1.5">Álbum</label>
            <input
              type="text"
              value={album}
              onChange={(e) => setAlbum(e.target.value)}
              placeholder="Opcional"
              className="w-full bg-bg-input border border-border-subtle text-text-primary text-body px-3 h-8 rounded-sm placeholder:text-text-tertiary focus:outline-none focus:border-accent transition-colors"
            />
          </div>
          <div className="flex gap-3">
            <div className="flex-1">
              <label className="block text-caption text-text-secondary mb-1.5">Género</label>
              <input
                type="text"
                value={genre}
                onChange={(e) => setGenre(e.target.value)}
                placeholder="Opcional"
                className="w-full bg-bg-input border border-border-subtle text-text-primary text-body px-3 h-8 rounded-sm placeholder:text-text-tertiary focus:outline-none focus:border-accent transition-colors"
              />
            </div>
            <div className="w-24">
              <label className="block text-caption text-text-secondary mb-1.5">Año</label>
              <input
                type="text"
                value={year}
                onChange={(e) => setYear(e.target.value.replace(/\D/g, "").slice(0, 4))}
                placeholder="Opcional"
                className="w-full bg-bg-input border border-border-subtle text-text-primary text-body px-3 h-8 rounded-sm placeholder:text-text-tertiary focus:outline-none focus:border-accent transition-colors"
              />
            </div>
          </div>
          {error && <p className="text-caption text-text-danger">{error}</p>}
        </div>
        <div className="flex justify-end gap-2 px-4 py-3 border-t border-border-subtle">
          <button
            onClick={onClose}
            className="bg-bg-input text-text-secondary px-4 h-8 text-caption rounded-sm border border-border-subtle hover:text-text-primary cursor-pointer transition-colors"
          >
            Cancelar
          </button>
          <button
            onClick={handleAdd}
            disabled={!audioPath || !title.trim() || isAdding}
            className="bg-accent text-bg-root px-4 h-8 text-caption font-medium rounded-sm hover:bg-accent-hover cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed transition-all"
          >
            {isAdding ? "Agregando..." : "Agregar"}
          </button>
        </div>
      </div>
    </div>
  );
}
