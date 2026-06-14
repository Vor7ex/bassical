import { open } from "@tauri-apps/plugin-dialog";

export async function pickAudioFile(): Promise<string | null> {
  const selected = await open({
    multiple: false,
    filters: [{ name: "Audio", extensions: ["mp3", "wav", "flac", "ogg"] }],
  });
  if (typeof selected === "string") return selected;
  return null;
}
