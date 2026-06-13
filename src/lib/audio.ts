import { invoke } from "@tauri-apps/api/core";

export async function initApp(): Promise<string> {
  return await invoke<string>("init_app");
}
