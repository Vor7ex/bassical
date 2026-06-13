import { pickAudioFile } from "@/lib/dialog";
import { reassignAudioPath } from "@/lib/library";
import { useLibraryStore } from "@/lib/store";
import type { Song } from "@/lib/types";
import { useState } from "react";

interface MissingFileDialogProps {
  song: Song;
  onClose: () => void;
}

export function MissingFileDialog({ song, onClose }: MissingFileDialogProps) {
  const [isReassigning, setIsReassigning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const updateSongInStore = useLibraryStore((s) => s.updateSongInStore);
  const removeSongFromStore = useLibraryStore((s) => s.removeSongFromStore);

  async function handleReassign() {
    setIsReassigning(true);
    setError(null);
    const newPath = await pickAudioFile();
    if (!newPath) {
      setIsReassigning(false);
      return;
    }
    try {
      const updated = await reassignAudioPath(song.id, newPath);
      updateSongInStore(updated);
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setIsReassigning(false);
    }
  }

  function handleDelete() {
    removeSongFromStore(song.id);
    onClose();
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
          if (e.key === "Escape") onClose();
        }}
      >
        <div className="bg-bg-surface px-4 h-9 flex items-center border-b border-border-subtle rounded-t-md">
          <span className="text-caption text-text-primary">Archivo no encontrado</span>
        </div>
        <div className="p-4 space-y-3">
          <p className="text-body text-text-secondary leading-relaxed">
            El archivo de audio asociado a{" "}
            <strong className="text-text-primary">&ldquo;{song.title}&rdquo;</strong>
            {song.artist && (
              <>
                {" de "}
                <strong className="text-text-primary">{song.artist}</strong>
              </>
            )}{" "}
            ya no se encuentra en la ruta registrada:
          </p>
          <div className="bg-bg-input border border-danger-border px-3 py-2 text-mono text-xs text-text-danger break-all rounded-sm">
            {song.audioPath}
          </div>
          <p className="text-body text-text-tertiary leading-relaxed">
            El archivo puede haber sido movido o eliminado. Reasigne la ruta o elimine esta
            entrada.
          </p>
          {error && <p className="text-caption text-text-danger">{error}</p>}
        </div>
        <div className="flex justify-end gap-2 px-4 py-3 border-t border-border-subtle">
          <button
            onClick={handleDelete}
            className="bg-danger-bg text-text-danger px-4 h-8 text-caption rounded-sm border border-danger-border hover:opacity-80 cursor-pointer transition-opacity"
          >
            Eliminar entrada
          </button>
          <button
            onClick={handleReassign}
            disabled={isReassigning}
            className="bg-accent text-bg-root px-4 h-8 text-caption font-medium rounded-sm hover:bg-accent-hover cursor-pointer disabled:opacity-30 transition-all"
          >
            {isReassigning ? "Reasignando..." : "Reasignar ruta"}
          </button>
        </div>
      </div>
    </div>
  );
}
