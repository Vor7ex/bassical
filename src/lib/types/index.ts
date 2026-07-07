// Tipos principales de la aplicación

export interface Song {
  id: string;
  title: string;
  artist: string | null;
  album: string | null;
  genre: string | null;
  year: number | null;
  tuning: string | null;
  bpm: number | null;
  difficulty: number | null;
  tags: string[];
  audioPath: string;
  audioMissing: boolean;
  hasTab: boolean;
  hasCalibration: boolean;
  preferredSpeed: number;
  lastPositionMs: number;
  createdAt: string;
  updatedAt: string;
}

export interface Library {
  songs: Song[];
}

export interface TimingPoint {
  offsetMs: number;
  bpm: number;
  timeSignature?: TimeSignature;
}

export interface TimeSignature {
  numerator: number;
  denominator: number;
}

export interface AppConfig {
  audioDevice: string | null;
  bufferSize: number;
  theme: "light" | "dark";
}

// Esquema del archivo songs/<uuid>.bassical.json (ADR-003).
// `tab` y `practice` son opcionales y se omiten del JSON cuando no existen
// (Sprint 4 solo usa timingPoints; Sprint 5 rellenará tab/practice).
export interface BassicalTab {
  schemaVersion: 1;
  id: string;
  title: string;
  artist?: string | null;
  audioPath: string;
  timingPoints: TimingPoint[];
  tab?: TabData;
  practice?: PracticeData;
}

export interface TabData {
  strings: number;
  tuning: string[];
  measures: Measure[];
}

export interface Measure {
  timeSignature: TimeSignature;
  beats: Beat[];
}

export interface Beat {
  duration: string;
  notes: Note[];
}

export interface Note {
  string: number;
  fret: number;
  technique?: string | null;
}

export interface PracticeData {
  preferredSpeed: number;
  lastPositionMs: number;
}

export interface AudioInfo {
  durationMs: number;
  sampleRate: number;
  channels: number;
  peaks: number[];
}

export interface AudioState {
  audioInfo: AudioInfo | null;
  isLoaded: boolean;
  isLoading: boolean;
  error: string | null;
}
