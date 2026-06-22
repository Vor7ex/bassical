const ZOOM_THRESHOLD = 0.98;

interface ViewportPeaksParams {
  overviewPeaks: number[];
  viewportStartMs: number;
  viewportEndMs: number;
  durationMs: number;
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
  const { overviewPeaks, viewportStartMs, viewportEndMs, durationMs } = params;

  const viewportSpan = viewportEndMs - viewportStartMs;
  const isZoomedIn = durationMs > 0 && viewportSpan < durationMs * ZOOM_THRESHOLD;

  if (!isZoomedIn) return overviewPeaks;
  return sliceOverviewPeaks(overviewPeaks, viewportStartMs, viewportEndMs, durationMs);
}
