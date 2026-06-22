import type { TimingPoint, TimeSignature } from "@/lib/types";

export interface BeatLine {
  ms: number;
  isDownbeat: boolean;
  beatNumber: number;
  barNumber: number;
}

const DEFAULT_TIME_SIGNATURE: TimeSignature = { numerator: 4, denominator: 4 };

function effectiveTimeSignature(tp: TimingPoint): TimeSignature {
  return tp.timeSignature ?? DEFAULT_TIME_SIGNATURE;
}

interface SegmentParams {
  tp: TimingPoint;
  nextOffset: number;
  viewportStartMs: number;
  viewportEndMs: number;
  barOffset: number;
}

function generateBeatsForSegment(params: SegmentParams): { lines: BeatLine[]; beatCount: number } {
  const { tp, nextOffset, viewportStartMs, viewportEndMs, barOffset } = params;
  const ts = effectiveTimeSignature(tp);
  const beatMs = 60000 / tp.bpm;

  if (beatMs <= 0) {
    return { lines: [], beatCount: 0 };
  }

  const lines: BeatLine[] = [];

  const startBeatIndex = Math.max(
    0,
    Math.ceil((viewportStartMs - tp.offsetMs) / beatMs),
  );

  let segmentBeatIndex = startBeatIndex;

  while (tp.offsetMs + segmentBeatIndex * beatMs < nextOffset
    && tp.offsetMs + segmentBeatIndex * beatMs <= viewportEndMs) {

    const ms = tp.offsetMs + segmentBeatIndex * beatMs;
    const beatNumber = (segmentBeatIndex % ts.numerator) + 1;
    const barNumber = barOffset + 1 + Math.floor(segmentBeatIndex / ts.numerator);

    lines.push({
      ms,
      isDownbeat: beatNumber === 1,
      beatNumber,
      barNumber,
    });

    segmentBeatIndex++;
  }

  return { lines, beatCount: segmentBeatIndex };
}

export function computeBeatGrid(
  timingPoints: TimingPoint[],
  viewportStartMs: number,
  viewportEndMs: number,
  durationMs: number,
): BeatLine[] {
  if (timingPoints.length === 0) return [];

  const sorted = [...timingPoints].sort((a, b) => a.offsetMs - b.offsetMs);
  const lines: BeatLine[] = [];
  let barOffset = 0;

  for (let segIdx = 0; segIdx < sorted.length; segIdx++) {
    const tp = sorted[segIdx];
    const nextOffset =
      segIdx + 1 < sorted.length ? sorted[segIdx + 1].offsetMs : durationMs;

    const { lines: segmentLines, beatCount } = generateBeatsForSegment({
      tp,
      nextOffset,
      viewportStartMs,
      viewportEndMs,
      barOffset,
    });

    lines.push(...segmentLines);

    if (beatCount > 0) {
      const ts = effectiveTimeSignature(tp);
      barOffset += Math.ceil(beatCount / ts.numerator);
    }
  }

  return lines;
}
