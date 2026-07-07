export interface BpmEstimate {
  bpm: number | null;
  meanIntervalMs: number | null;
  accepted: number;
  rejected: number;
}

function median(arr: number[]): number {
  if (arr.length === 0) return 0;
  const sorted = [...arr].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 0
    ? (sorted[mid - 1] + sorted[mid]) / 2
    : sorted[mid];
}

function rejectOutliers(iois: number[]): {
  accepted: number[];
  rejected: number;
} {
  const med = median(iois);
  if (med <= 0) return { accepted: [], rejected: iois.length };

  const absDeviations = iois.map((v) => Math.abs(v - med));
  const mad = median(absDeviations);

  const accepted: number[] = [];
  let rejected = 0;

  for (const ioi of iois) {
    if (mad > 0) {
      const modifiedZ = 0.6745 * (ioi - med) / mad;
      if (Math.abs(modifiedZ) <= 3.5) {
        accepted.push(ioi);
      } else {
        rejected++;
      }
    } else {
      accepted.push(ioi);
    }
  }

  return { accepted, rejected };
}

export function estimateBpm(taps: number[], minTaps = 8): BpmEstimate {
  if (taps.length < minTaps) {
    return { bpm: null, meanIntervalMs: null, accepted: 0, rejected: 0 };
  }

  const iois: number[] = [];
  for (let i = 1; i < taps.length; i++) {
    iois.push(taps[i] - taps[i - 1]);
  }

  if (iois.length === 0) {
    return { bpm: null, meanIntervalMs: null, accepted: 0, rejected: 0 };
  }

  const { accepted: acceptedIois, rejected } = rejectOutliers(iois);

  if (acceptedIois.length === 0) {
    return { bpm: null, meanIntervalMs: null, accepted: 0, rejected };
  }

  const meanIntervalMs =
    acceptedIois.reduce((a, b) => a + b, 0) / acceptedIois.length;
  const bpm = 60000 / meanIntervalMs;

  return {
    bpm,
    meanIntervalMs,
    accepted: acceptedIois.length,
    rejected,
  };
}
