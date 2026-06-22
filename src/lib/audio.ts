import { invoke } from "@tauri-apps/api/core";

export interface AudioInfo {
  durationMs: number;
  sampleRate: number;
  channels: number;
  peaks: number[];
  complete: boolean;
}

export interface SongMetadata {
  title?: string;
  artist?: string;
  album?: string;
  year?: string;
  genre?: string;
}

interface LoadAudioParams {
  path: string;
  autoplay?: boolean;
}

interface DecodeAudioParams {
  path: string;
}

interface StartPlaybackParams {
  path: string;
  positionMs?: number;
}

interface PeaksInRangeParams {
  path: string;
  startMs: number;
  endMs: number;
  numBins: number;
}

interface ExtractMetadataParams {
  filePath: string;
}

export async function loadAudio(params: LoadAudioParams): Promise<AudioInfo> {
  return await invoke<AudioInfo>("load_audio", { ...params });
}

export async function decodeAudio(params: DecodeAudioParams): Promise<AudioInfo> {
  return await invoke<AudioInfo>("decode_audio", { ...params });
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

export async function startPlayback(params: StartPlaybackParams): Promise<AudioInfo> {
  return await invoke<AudioInfo>("start_playback", { ...params });
}

export async function stopPlayback(): Promise<void> {
  return await invoke<void>("stop_playback");
}

export async function activateFullBufferPlayback(): Promise<void> {
  return await invoke<void>("activate_full_buffer_playback");
}

export async function isFullBufferReady(): Promise<boolean> {
  return await invoke<boolean>("is_full_buffer_ready");
}

export async function getPeaksInRange(params: PeaksInRangeParams): Promise<number[]> {
  return await invoke<number[]>("get_peaks_in_range", {
    path: params.path,
    range: { startMs: params.startMs, endMs: params.endMs, numBins: params.numBins },
  });
}

export async function extractMetadata(params: ExtractMetadataParams): Promise<SongMetadata> {
  return await invoke<SongMetadata>("extract_metadata", { ...params });
}
