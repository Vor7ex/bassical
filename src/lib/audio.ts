import { invoke } from "@tauri-apps/api/core";

export interface AudioInfo {
  durationMs: number;
  sampleRate: number;
  channels: number;
  peaks: number[];
}

export async function loadAudio(path: string): Promise<AudioInfo> {
  return await invoke<AudioInfo>("load_audio", { path });
}

export async function getDecodeProgress(): Promise<number> {
  return await invoke<number>("get_decode_progress");
}

export async function getDecodedPeaks(): Promise<number[]> {
  return await invoke<number[]>("get_decoded_peaks");
}

export async function playAudio(): Promise<void> {
  return await invoke<void>("play_audio");
}

export async function pauseAudio(): Promise<void> {
  return await invoke<void>("pause_audio");
}

export async function seekAudio(positionMs: number): Promise<void> {
  return await invoke<void>("seek_audio", { positionMs });
}

export async function setPlaybackSpeed(speed: number): Promise<void> {
  return await invoke<void>("set_playback_speed", { speed });
}

export async function getAudioPosition(): Promise<number> {
  return await invoke<number>("get_audio_position");
}

export async function getAudioDuration(): Promise<number> {
  return await invoke<number>("get_audio_duration");
}

export async function isAudioPlaying(): Promise<boolean> {
  return await invoke<boolean>("is_audio_playing");
}

export async function cacheAudio(path: string): Promise<void> {
  return await invoke<void>("cache_audio", { path });
}
