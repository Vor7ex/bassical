import { invoke } from "@tauri-apps/api/core";
import type { Song, Library } from "@/lib/types";

export async function initApp(): Promise<string> {
  return await invoke<string>("init_app");
}

export async function getLibrary(): Promise<Library> {
  return await invoke<Library>("get_library");
}

export async function getLibraryWithStatus(): Promise<Library> {
  return await invoke<Library>("get_library_with_status");
}

export async function addSong(
  title: string,
  artist: string | undefined,
  audioPath: string,
  album?: string,
  genre?: string,
  year?: number,
): Promise<Song> {
  return await invoke<Song>("add_song", {
    title,
    artist: artist ?? null,
    audioPath,
    album: album ?? null,
    genre: genre ?? null,
    year: year ?? null,
  });
}

export async function updateSong(
  id: string,
  fields: {
    title?: string;
    artist?: string;
    album?: string;
    genre?: string;
    year?: number;
    tuning?: string;
    bpm?: number;
    difficulty?: number;
    tags?: string[];
    audioPath?: string;
  },
): Promise<Song> {
  return await invoke<Song>("update_song", {
    id,
    update: {
      title: fields.title ?? null,
      artist: fields.artist ?? null,
      album: fields.album ?? null,
      genre: fields.genre ?? null,
      year: fields.year ?? null,
      tuning: fields.tuning ?? null,
      bpm: fields.bpm ?? null,
      difficulty: fields.difficulty ?? null,
      tags: fields.tags ?? null,
      audioPath: fields.audioPath ?? null,
    },
  });
}

export async function deleteSong(id: string): Promise<void> {
  return await invoke<void>("delete_song", { id });
}

export async function checkAudioExists(id: string): Promise<boolean> {
  return await invoke<boolean>("check_audio_exists", { id });
}

export async function reassignAudioPath(
  id: string,
  newPath: string,
): Promise<Song> {
  return await invoke<Song>("reassign_audio_path", { id, newPath });
}
