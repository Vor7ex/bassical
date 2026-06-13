// Tipos principales de la aplicación

export interface Song {
  id: string;
  title: string;
  artist: string | null;
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
}

export interface AppConfig {
  audioDevice: string | null;
  bufferSize: number;
  theme: "light" | "dark";
}

// Esquema del archivo .bassical.json
export interface BassicalTab {
  schemaVersion: 1;
  id: string;
  title: string;
  artist: string | null;
  audioPath: string;
  timingPoints: TimingPoint[];
  tab: TabData;
  practice: PracticeData;
}

export interface TabData {
  strings: 4;
  tuning: ["E", "A", "D", "G"];
  measures: Measure[];
}

export interface Measure {
  timeSignature: [number, number];
  beats: Beat[];
}

export interface Beat {
  duration: string;
  notes: Note[];
}

export interface Note {
  string: number;
  fret: number;
  technique: string | null;
}

export interface PracticeData {
  preferredSpeed: number;
  lastPositionMs: number;
}
