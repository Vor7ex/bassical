import { invoke } from "@tauri-apps/api/core";
import type { TimingPoint } from "@/lib/types";

export async function toggleMetronome(): Promise<boolean> {
  return await invoke<boolean>("toggle_metronome");
}

export async function getMetronomeState(): Promise<boolean> {
  return await invoke<boolean>("get_metronome_state");
}

export async function setMetronomeGrid(
  timingPoints: TimingPoint[],
  durationMs: number,
): Promise<void> {
  return await invoke<void>("set_metronome_grid", {
    timingPoints,
    durationMs,
  });
}

export async function setMetronomeBalance(balance: number): Promise<void> {
  return await invoke<void>("set_metronome_balance", { balance });
}

export async function getMetronomeBalance(): Promise<number> {
  return await invoke<number>("get_metronome_balance");
}