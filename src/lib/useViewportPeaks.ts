import { useEffect, useState } from "react";
import { getPeaksInRange } from "@/lib/audio";

const PEAKS_FETCH_DEBOUNCE_MS = 80;
const ZOOM_THRESHOLD = 0.98;
const NUM_BINS = 2000;

interface ViewportPeaksParams {
  audioPath: string;
  overviewPeaks: number[];
  viewportStartMs: number;
  viewportEndMs: number;
  durationMs: number;
  fullBufferReady: boolean;
}

function useDetailedPeaks(
  audioPath: string,
  startMs: number,
  endMs: number,
  enabled: boolean,
): number[] {
  const [peaks, setPeaks] = useState<number[]>([]);

  useEffect(() => {
    if (!enabled) {
      setPeaks([]);
      return;
    }
    let cancelled = false;
    const timer = setTimeout(() => {
      void getPeaksInRange({ path: audioPath, startMs, endMs, numBins: NUM_BINS })
        .then((result) => {
          if (!cancelled) setPeaks(result);
        })
        .catch((err) => {
          if (!cancelled) console.error("[useViewportPeaks] fetch failed:", err);
        });
    }, PEAKS_FETCH_DEBOUNCE_MS);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [audioPath, startMs, endMs, enabled]);

  return peaks;
}

function sliceOverviewPeaks(
  overviewPeaks: number[],
  startMs: number,
  endMs: number,
  durationMs: number,
): number[] {
  if (overviewPeaks.length === 0 || durationMs <= 0) return overviewPeaks;
  const idxStart = Math.floor((startMs / durationMs) * overviewPeaks.length);
  const idxEnd = Math.ceil((endMs / durationMs) * overviewPeaks.length);
  return overviewPeaks.slice(idxStart, Math.max(idxEnd, idxStart + 1));
}

export function useViewportPeaks(params: ViewportPeaksParams): number[] {
  const {
    audioPath,
    overviewPeaks,
    viewportStartMs,
    viewportEndMs,
    durationMs,
    fullBufferReady,
  } = params;

  const viewportSpan = viewportEndMs - viewportStartMs;
  const isZoomedIn = durationMs > 0 && viewportSpan < durationMs * ZOOM_THRESHOLD;
  const detailedPeaks = useDetailedPeaks(
    audioPath,
    viewportStartMs,
    viewportEndMs,
    isZoomedIn && fullBufferReady,
  );

  if (!isZoomedIn) return overviewPeaks;
  if (detailedPeaks.length > 0) return detailedPeaks;
  return sliceOverviewPeaks(overviewPeaks, viewportStartMs, viewportEndMs, durationMs);
}
