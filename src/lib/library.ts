import { invoke } from "@tauri-apps/api/core";
import type { Song, Library } from "@/lib/types";

export async function initApp(): Promise<string> {
  return await invoke<string>("init_app");
}

export async function getLibrary(): Promise<Library> {
  return await invoke<Library>("get_library");
}

export async function addSong(
  title: string,
  artist: string | undefined,
  audioPath: string,
): Promise<Song> {
  return await invoke<Song>("add_song", {
    title,
    artist: artist ?? null,
    audioPath,
  });
}

export async function updateSong(
  id: string,
  title?: string,
  artist?: string,
  audioPath?: string,
): Promise<Song> {
  return await invoke<Song>("update_song", {
    id,
    title: title ?? null,
    artist: artist ?? null,
    audioPath: audioPath ?? null,
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
