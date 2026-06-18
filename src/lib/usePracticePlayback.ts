import { useEffect, useRef, useCallback, useState } from "react";
import {
  decodeAudio,
  startPlayback,
  stopPlayback,
  pauseAudio,
  seekAudio,
  setPlaybackSpeed,
  getAudioPosition,
  getDecodeProgress,
  getDecodedPeaks,
} from "@/lib/audio";

interface AudioState {
  durationMs: number;
  sampleRate: number;
  channels: number;
  peaks: number[];
}

function stopInterval(ref: ReturnType<typeof setInterval> | null) {
  if (ref) clearInterval(ref);
}

export function usePracticePlayback(audioPath: string) {
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentPositionMs, setCurrentPositionMs] = useState(0);
  const [audioState, setAudioState] = useState<AudioState | null>(null);
  const [decodeProgress, setDecodeProgress] = useState(0);
  const [playbackSpeed, setPlaybackSpeedState] = useState(1.0);

  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const decodePollRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const lastPeakUpdateRef = useRef(0);
  const loadedPathRef = useRef("");
  const decodeFinalizedRef = useRef(false);

  useEffect(() => {
    lastPeakUpdateRef.current = 0;
    setDecodeProgress(0);
    decodeFinalizedRef.current = false;

    if (!audioPath) {
      setAudioState(null);
      return;
    }

    if (loadedPathRef.current !== audioPath) {
      setCurrentPositionMs(0);
      loadedPathRef.current = audioPath;
    }

    decodeAudio(audioPath)
      .then((info) => {
        setAudioState(info);
        startDecodePolling();
      })
      .catch(() => setAudioState(null));

    return () => {
      stopInterval(decodePollRef.current);
      decodePollRef.current = null;
    };
  }, [audioPath]);

  useEffect(
    () => () => {
      pauseAudio().catch(() => {});
      stopPlayback().catch(() => {});
    },
    [],
  );

  function startDecodePolling() {
    stopInterval(decodePollRef.current);
    decodePollRef.current = setInterval(tickDecodePoll, 200);
  }

  async function tickDecodePoll() {
    try {
      const progress = await getDecodeProgress();
      setDecodeProgress(progress);
      await maybeRefreshPeaks(progress);
      if (progress >= 1.0) await finalizeDecode();
    } catch {
      stopInterval(decodePollRef.current);
      decodePollRef.current = null;
    }
  }

  async function maybeRefreshPeaks(progress: number) {
    if (decodeFinalizedRef.current) return;
    const shouldRefresh = progress - lastPeakUpdateRef.current > 0.02;
    if (!shouldRefresh) return;
    lastPeakUpdateRef.current = progress;
    const peaks = await getDecodedPeaks();
    if (peaks.length > 0) {
      setAudioState((prev) => (prev ? { ...prev, peaks } : null));
    }
  }

  async function finalizeDecode() {
    decodeFinalizedRef.current = true;
    stopInterval(decodePollRef.current);
    decodePollRef.current = null;
    const peaks = await getDecodedPeaks();
    if (peaks.length > 0) {
      setAudioState((prev) => (prev ? { ...prev, peaks } : null));
    }
    setDecodeProgress(1.0);
  }

  useEffect(() => {
    if (!isPlaying) {
      stopInterval(pollRef.current);
      pollRef.current = null;
      return;
    }
    pollRef.current = setInterval(() => {
      getAudioPosition()
        .then(setCurrentPositionMs)
        .catch(() => {});
    }, 100);
    return () => {
      stopInterval(pollRef.current);
      pollRef.current = null;
    };
  }, [isPlaying]);

  const handlePlayPause = useCallback(async () => {
    try {
      if (isPlaying) {
        await pauseAudio();
        setIsPlaying(false);
      } else {
        const info = await startPlayback(audioPath);
        setAudioState((prev) =>
          prev
            ? {
                ...prev,
                durationMs: info.durationMs,
                sampleRate: info.sampleRate,
                channels: info.channels,
              }
            : info,
        );
        setIsPlaying(true);
      }
    } catch (err) {
      console.error("Error de reproducción:", err);
    }
  }, [isPlaying, audioPath]);

  const handleSeek = useCallback(async (positionMs: number) => {
    setCurrentPositionMs(positionMs);
    await seekAudio(positionMs);
  }, []);

  const handleSpeedChange = useCallback(async (speed: number) => {
    try {
      await setPlaybackSpeed(speed);
      setPlaybackSpeedState(speed);
    } catch (err) {
      console.error("Error cambiando velocidad:", err);
    }
  }, []);

  return {
    isPlaying,
    playbackSpeed,
    currentPositionMs,
    audioState,
    decodeProgress,
    handlePlayPause,
    handleSeek,
    handleSpeedChange,
  };
}
