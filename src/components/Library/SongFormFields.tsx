import { StarRating } from "./StarRating";
import { TagInput } from "./TagInput";

const TUNINGS = [
  "E Standard (EADG)",
  "Drop D (DADG)",
  "D Standard (DGCF)",
  "Drop C (CGCF)",
  "C Standard (CFBbEb)",
  "B Standard (BEAD)",
];

interface SongFormFieldsProps {
  title: string;
  setTitle: (v: string) => void;
  artist: string;
  setArtist: (v: string) => void;
  album: string;
  setAlbum: (v: string) => void;
  genre: string;
  setGenre: (v: string) => void;
  year: string;
  setYear: (v: string) => void;
  tuning: string;
  setTuning: (v: string) => void;
  bpm: string;
  setBpm: (v: string) => void;
  difficulty: number | null;
  setDifficulty: (v: number | null) => void;
  tags: string[];
  setTags: (v: string[]) => void;
  audioPath?: string;
  onBrowseAudioPath?: () => void;
}

const inputClass =
  "w-full bg-bg-input border border-border-subtle text-text-primary text-body px-3 h-8 rounded-sm focus:outline-none focus:border-accent transition-colors";
const inputClassPlaceholder =
  "w-full bg-bg-input border border-border-subtle text-text-primary text-body px-3 h-8 rounded-sm placeholder:text-text-tertiary focus:outline-none focus:border-accent transition-colors";

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div>
      <label className="block text-caption text-text-secondary mb-1.5">{label}</label>
      {children}
    </div>
  );
}

export function SongFormFields(props: SongFormFieldsProps) {
  return (
    <>
      <div className="grid grid-cols-2 gap-3">
        <Field label="Título">
          <input
            type="text" value={props.title}
            onChange={(e) => props.setTitle(e.target.value)}
            className={inputClass} autoFocus
          />
        </Field>
        <Field label="Artista">
          <input
            type="text" value={props.artist}
            onChange={(e) => props.setArtist(e.target.value)}
            placeholder="Opcional" className={inputClassPlaceholder}
          />
        </Field>
        <Field label="Álbum">
          <input
            type="text" value={props.album}
            onChange={(e) => props.setAlbum(e.target.value)}
            placeholder="Opcional" className={inputClassPlaceholder}
          />
        </Field>
        <Field label="Género">
          <input
            type="text" value={props.genre}
            onChange={(e) => props.setGenre(e.target.value)}
            placeholder="Rock, Metal, Jazz..." className={inputClassPlaceholder}
          />
        </Field>
        <Field label="Año">
          <input
            type="number" value={props.year}
            onChange={(e) => props.setYear(e.target.value)}
            placeholder="2024" min={1900} max={2099}
            className={inputClassPlaceholder}
          />
        </Field>
        <Field label="Afinación">
          <select
            value={props.tuning}
            onChange={(e) => props.setTuning(e.target.value)}
            className={`${inputClass} cursor-pointer`}
          >
            <option value="">Sin especificar</option>
            {TUNINGS.map((t) => (
              <option key={t} value={t}>{t}</option>
            ))}
          </select>
        </Field>
        <Field label="BPM">
          <input
            type="number" value={props.bpm}
            onChange={(e) => props.setBpm(e.target.value)}
            placeholder="120" min={20} max={400}
            className={inputClassPlaceholder}
          />
        </Field>
        <Field label="Dificultad">
          <div className="h-8 flex items-center">
            <StarRating value={props.difficulty} onChange={props.setDifficulty} />
          </div>
        </Field>
      </div>
      <Field label="Etiquetas">
        <TagInput tags={props.tags} onChange={props.setTags} />
      </Field>
      {props.audioPath !== undefined && (
        <Field label="Ruta del archivo">
          <div className="flex gap-2">
            <input
              type="text" value={props.audioPath} readOnly
              className="flex-1 bg-bg-input border border-border-subtle text-text-secondary text-body px-3 h-8 rounded-sm truncate"
              title={props.audioPath}
            />
            {props.onBrowseAudioPath && (
              <button
                onClick={props.onBrowseAudioPath}
                type="button"
                className="bg-bg-input border border-border-subtle text-text-secondary text-caption px-3 h-8 rounded-sm hover:text-text-primary cursor-pointer transition-colors shrink-0"
              >
                Examinar
              </button>
            )}
          </div>
        </Field>
      )}
    </>
  );
}
