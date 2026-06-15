import { useEffect, useRef, useCallback } from "react";
import { useSessionStore } from "@/lib/store";
import {
  loadAudio,
  playAudio,
  pauseAudio,
  seekAudio,
  setPlaybackSpeed,
  getAudioPosition,
  getDecodeProgress,
  getDecodedPeaks,
} from "@/lib/audio";

export function useAudioPlayback(audioPath: string) {
  const isPlaying = useSessionStore((s) => s.isPlaying);
  const playbackSpeed = useSessionStore((s) => s.playbackSpeed);
  const currentPositionMs = useSessionStore((s) => s.currentPositionMs);
  const audioState = useSessionStore((s) => s.audioState);
  const decodeProgress = useSessionStore((s) => s.decodeProgress);
  const setIsPlaying = useSessionStore((s) => s.setIsPlaying);
  const setPlaybackSpeedStore = useSessionStore((s) => s.setPlaybackSpeed);
  const setCurrentPositionMs = useSessionStore((s) => s.setCurrentPositionMs);
  const setAudioState = useSessionStore((s) => s.setAudioState);
  const updatePeaks = useSessionStore((s) => s.updatePeaks);
  const setDecodeProgress = useSessionStore((s) => s.setDecodeProgress);

  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const decodePollRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const lastPeakUpdateRef = useRef(0);
  const loadedPathRef = useRef("");
  const prevIsPlayingRef = useRef(isPlaying);

  useEffect(() => {
    lastPeakUpdateRef.current = 0;
    setDecodeProgress(0);

    if (!audioPath) {
      setAudioState(null);
      return;
    }

    const isNewSong = loadedPathRef.current !== audioPath;
    if (isNewSong) {
      setCurrentPositionMs(0);
      loadedPathRef.current = audioPath;
    }

    loadAudio(audioPath)
      .then((info) => {
        setAudioState(info);
        startDecodePolling();
        if (useSessionStore.getState().isPlaying) {
          playAudio().catch(() => {});
        }
      })
      .catch((err) => {
        console.error("Error cargando audio:", err);
        setAudioState(null);
      });

    return () => {
      if (pollRef.current) clearInterval(pollRef.current);
      if (decodePollRef.current) clearInterval(decodePollRef.current);
    };
  }, [audioPath, setAudioState, setDecodeProgress]);

  useEffect(() => {
    if (prevIsPlayingRef.current === isPlaying) return;
    prevIsPlayingRef.current = isPlaying;
    if (isPlaying) {
      playAudio().catch(() => {});
    } else {
      pauseAudio().catch(() => {});
    }
  }, [isPlaying]);

  function startDecodePolling() {
    if (decodePollRef.current) clearInterval(decodePollRef.current);

    decodePollRef.current = setInterval(() => {
      getDecodeProgress()
        .then(async (progress) => {
          setDecodeProgress(progress);
          if (progress > 0.1 && progress - lastPeakUpdateRef.current > 0.05) {
            lastPeakUpdateRef.current = progress;
            await refreshPeaks();
          }
          if (progress >= 1.0) {
            await finishDecoding();
          }
        })
        .catch(() => stopDecodePolling());
    }, 200);
  }

  function stopDecodePolling() {
    if (decodePollRef.current) {
      clearInterval(decodePollRef.current);
      decodePollRef.current = null;
    }
  }

  async function refreshPeaks() {
    const peaks = await getDecodedPeaks();
    if (peaks.length > 0) updatePeaks(peaks);
  }

  async function finishDecoding() {
    stopDecodePolling();
    await refreshPeaks();
    setDecodeProgress(1.0);
  }

  useEffect(() => {
    if (isPlaying) {
      pollRef.current = setInterval(() => {
        getAudioPosition()
          .then((pos) => setCurrentPositionMs(pos))
          .catch(() => {});
      }, 100);
    } else if (pollRef.current) {
      clearInterval(pollRef.current);
      pollRef.current = null;
    }

    return () => {
      if (pollRef.current) {
        clearInterval(pollRef.current);
        pollRef.current = null;
      }
    };
  }, [isPlaying, setCurrentPositionMs]);

  const handlePlayPause = useCallback(async () => {
    try {
      if (isPlaying) {
        await pauseAudio();
        setIsPlaying(false);
      } else {
        await playAudio();
        setIsPlaying(true);
      }
    } catch (err) {
      console.error("Error de reproducción:", err);
    }
  }, [isPlaying, setIsPlaying]);

  const handleSeek = useCallback(
    async (positionMs: number) => {
      try {
        await seekAudio(positionMs);
        setCurrentPositionMs(positionMs);
      } catch (err) {
        console.error("Error de seek:", err);
      }
    },
    [setCurrentPositionMs],
  );

  const handleSpeedChange = useCallback(
    async (speed: number) => {
      try {
        await setPlaybackSpeed(speed);
        setPlaybackSpeedStore(speed);
      } catch (err) {
        console.error("Error cambiando velocidad:", err);
      }
    },
    [setPlaybackSpeedStore],
  );

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
