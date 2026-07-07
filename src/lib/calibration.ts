import { invoke } from "@tauri-apps/api/core";
import type { TimingPoint } from "@/lib/types";

export async function getCalibration(songId: string): Promise<TimingPoint[]> {
  return await invoke<TimingPoint[]>("get_calibration", { songId });
}

export async function saveTimingPoints(
  songId: string,
  timingPoints: TimingPoint[],
): Promise<void> {
  return await invoke<void>("save_timing_points", { songId, timingPoints });
}

export async function clearCalibration(songId: string): Promise<void> {
  return await invoke<void>("clear_calibration", { songId });
}

export async function recordTap(): Promise<number> {
  return await invoke<number>("record_calibration_tap");
}
