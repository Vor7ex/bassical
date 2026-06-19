import { useState } from "react";
import { updateSong } from "@/lib/library";
import { pickAudioFile } from "@/lib/dialog";
import { useLibraryStore } from "@/lib/store";
import type { Song } from "@/lib/types";
import { SongFormFields } from "./SongFormFields";

interface EditSongDialogProps {
  song: Song;
  onClose: () => void;
}

export function EditSongDialog({ song, onClose }: EditSongDialogProps) {
  const [title, setTitle] = useState(song.title);
  const [artist, setArtist] = useState(song.artist ?? "");
  const [album, setAlbum] = useState(song.album ?? "");
  const [genre, setGenre] = useState(song.genre ?? "");
  const [year, setYear] = useState(song.year?.toString() ?? "");
  const [tuning, setTuning] = useState(song.tuning ?? "");
  const [bpm, setBpm] = useState(song.bpm?.toString() ?? "");
  const [difficulty, setDifficulty] = useState<number | null>(song.difficulty);
  const [tags, setTags] = useState<string[]>(song.tags);
  const [audioPath, setAudioPath] = useState(song.audioPath);
  const [error, setError] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const updateSongInStore = useLibraryStore((s) => s.updateSongInStore);

  async function handleBrowsePath() {
    const picked = await pickAudioFile();
    if (picked) setAudioPath(picked);
  }

  async function handleSave() {
    if (!title.trim()) {
      setError("El título es obligatorio");
      return;
    }
    setIsSaving(true);
    setError(null);
    try {
      const updated = await updateSong(song.id, {
        title: title.trim(),
        artist: artist.trim() || undefined,
        album: album.trim() || undefined,
        genre: genre.trim() || undefined,
        year: year ? parseInt(year, 10) : undefined,
        tuning: tuning || undefined,
        bpm: bpm ? parseFloat(bpm) : undefined,
        difficulty: difficulty ?? undefined,
        tags,
        audioPath: audioPath !== song.audioPath ? audioPath : undefined,
      });
      updateSongInStore(updated);
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <div
      className="fixed inset-0 bg-black/60 flex items-center justify-center z-40"
      onMouseDown={onClose}
    >
      <div
        className="bg-bg-raised border border-border-subtle w-[560px] rounded-md shadow-modal"
        onClick={(e) => e.stopPropagation()}
        onMouseDown={(e) => e.stopPropagation()}
        onKeyDown={(e) => {
          e.stopPropagation();
          if (e.key === "Escape") onClose();
        }}
      >
        <div className="bg-bg-surface px-4 h-9 flex items-center border-b border-border-subtle rounded-t-md">
          <span className="text-caption text-text-primary">Editar canción</span>
        </div>
        <div className="p-4 space-y-3">
          <SongFormFields
            title={title} setTitle={setTitle}
            artist={artist} setArtist={setArtist}
            album={album} setAlbum={setAlbum}
            genre={genre} setGenre={setGenre}
            year={year} setYear={setYear}
            tuning={tuning} setTuning={setTuning}
            bpm={bpm} setBpm={setBpm}
            difficulty={difficulty} setDifficulty={setDifficulty}
            tags={tags} setTags={setTags}
            audioPath={audioPath} onBrowseAudioPath={handleBrowsePath}
          />
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
            onClick={handleSave}
            disabled={!title.trim() || isSaving}
            className="bg-accent text-bg-root px-4 h-8 text-caption font-medium rounded-sm hover:bg-accent-hover cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed transition-all"
          >
            {isSaving ? "Guardando..." : "Guardar"}
          </button>
        </div>
      </div>
    </div>
  );
}
