import { useCallback, useEffect } from "react";
import { useCalibrationStore } from "@/lib/store";
import { recordTap } from "@/lib/calibration";
import { estimateBpm } from "@/lib/tapBpm";
import type { TimingPoint } from "@/lib/types";

function commitReady(tp: TimingPoint | null, bpm: number | null, tapCount: number): boolean {
  return tp !== null && bpm !== null && tapCount >= 8;
}

function useCalibrationSelectors() {
  return {
    calibratingTpIndex: useCalibrationStore((s) => s.calibratingTpIndex),
    calibratingTp: useCalibrationStore((s) => {
      if (s.calibratingTpIndex === null) return null;
      return s.timingPoints[s.calibratingTpIndex] ?? null;
    }),
    tapPositions: useCalibrationStore((s) => s.tapPositions),
    startCalibrating: useCalibrationStore((s) => s.startCalibrating),
    stopCalibrating: useCalibrationStore((s) => s.stopCalibrating),
    pushTap: useCalibrationStore((s) => s.pushTap),
    updateTimingPoint: useCalibrationStore((s) => s.updateTimingPoint),
    setDetectedBpm: useCalibrationStore((s) => s.setDetectedBpm),
  };
}

export function useTapCalibration(
  handleSeek: (positionMs: number) => void,
  handlePlayPause: () => void,
  isPlaying: boolean,
) {
  const sel = useCalibrationSelectors();
  const est = estimateBpm(sel.tapPositions);

  useEffect(() => {
    sel.setDetectedBpm(est.bpm);
  }, [est.bpm, sel.setDetectedBpm]);

  const beginCalibrating = useCallback(
    (index: number) => {
      const tp = useCalibrationStore.getState().timingPoints[index];
      if (!tp) return;
      sel.startCalibrating(index);
      handleSeek(tp.offsetMs);
    },
    [sel, handleSeek],
  );

  const handleFirstTap = useCallback(async () => {
    if (sel.calibratingTpIndex === null) return;
    if (!isPlaying) handlePlayPause();
    const pos = await recordTap();
    sel.pushTap(pos);
  }, [sel.calibratingTpIndex, isPlaying, handlePlayPause, sel.pushTap]);

  const commitTapPoint = useCallback(() => {
    if (!commitReady(sel.calibratingTp, est.bpm, sel.tapPositions.length)) return;
    sel.updateTimingPoint(sel.calibratingTpIndex!, { bpm: est.bpm! });
    sel.stopCalibrating();
  }, [sel, est.bpm]);

  const canCommit = commitReady(sel.calibratingTp, est.bpm, sel.tapPositions.length);

  return {
    calibratingTpIndex: sel.calibratingTpIndex,
    calibratingTp: sel.calibratingTp,
    detectedBpm: est.bpm,
    tapCount: sel.tapPositions.length,
    acceptedCount: est.accepted,
    rejectedCount: est.rejected,
    canCommit,
    beginCalibrating,
    handleFirstTap,
    commitTapPoint,
    cancelCalibrating: sel.stopCalibrating,
  };
}
