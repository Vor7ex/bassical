import { useEffect, useRef, useCallback } from "react";

interface WaveformViewProps {
  peaks: number[];
  currentPositionMs: number;
  viewportStartMs: number;
  viewportEndMs: number;
  durationMs: number;
  onSeek: (positionMs: number) => void;
  onZoom?: (factor: number, centerMs: number) => void;
  onPan?: (deltaMs: number) => void;
  height?: number;
}

interface CanvasCtx {
  ctx: CanvasRenderingContext2D;
  width: number;
  height: number;
  dpr: number;
  peaks: number[];
  positionMs: number;
  viewportStartMs: number;
  viewportEndMs: number;
}

const DRAG_THRESHOLD_PX = 4;
const ZOOM_FACTOR = 1.25;

function drawBars(c: CanvasCtx) {
  const h = c.height * c.dpr;
  const barWidth = 2 * c.dpr;
  const gap = 1 * c.dpr;
  const totalBarWidth = barWidth + gap;
  const numBars = Math.floor(c.width / totalBarWidth);
  const centerY = h / 2;
  const peakCount = c.peaks.length;

  c.ctx.fillStyle = "oklch(0.45 0.10 155)";
  for (let i = 0; i < numBars; i++) {
    const peakIdx = Math.min(
      Math.floor((i / numBars) * peakCount),
      peakCount - 1,
    );
    const amplitude = c.peaks[peakIdx] * (h * 0.45);
    const x = i * totalBarWidth;
    c.ctx.fillRect(x, centerY - amplitude, barWidth, amplitude);
    c.ctx.fillRect(x, centerY, barWidth, amplitude);
  }
}

function drawPlayhead(c: CanvasCtx) {
  const h = c.height * c.dpr;
  const span = c.viewportEndMs - c.viewportStartMs;
  if (span <= 0) return;

  if (c.positionMs < c.viewportStartMs || c.positionMs > c.viewportEndMs) {
    return;
  }

  const ratio = (c.positionMs - c.viewportStartMs) / span;
  const playheadX = ratio * c.width;
  c.ctx.fillStyle = "oklch(0.72 0.18 155)";
  c.ctx.fillRect(playheadX - 1 * c.dpr, 0, 2 * c.dpr, h);
}

function renderFrame(c: CanvasCtx) {
  c.ctx.clearRect(0, 0, c.width, c.height);
  if (c.peaks.length === 0) return;
  drawBars(c);
  drawPlayhead(c);
}

export function WaveformView({
  peaks,
  currentPositionMs,
  viewportStartMs,
  viewportEndMs,
  durationMs,
  onSeek,
  onZoom,
  onPan,
  height = 200,
}: WaveformViewProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const peaksRef = useRef(peaks);
  peaksRef.current = peaks;

  const dragStateRef = useRef<{
    startX: number;
    lastX: number;
    dragged: boolean;
    viewportStartMs: number;
  } | null>(null);

  const onZoomRef = useRef(onZoom);
  onZoomRef.current = onZoom;
  const viewportRef = useRef({ start: viewportStartMs, end: viewportEndMs, duration: durationMs });
  viewportRef.current = { start: viewportStartMs, end: viewportEndMs, duration: durationMs };

  const draw = useCallback(
    (posMs: number) => {
      const canvas = canvasRef.current;
      if (!canvas) return;
      const ctx = canvas.getContext("2d");
      if (!ctx) return;
      const dpr = window.devicePixelRatio || 1;
      renderFrame({
        ctx,
        width: canvas.width,
        height,
        dpr,
        peaks: peaksRef.current,
        positionMs: posMs,
        viewportStartMs,
        viewportEndMs,
      });
    },
    [height, viewportStartMs, viewportEndMs],
  );

  useEffect(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container) return;

    const resize = () => {
      const { width } = container.getBoundingClientRect();
      const dpr = window.devicePixelRatio || 1;
      canvas.width = width * dpr;
      canvas.height = height * dpr;
      canvas.style.width = `${width}px`;
      canvas.style.height = `${height}px`;
      draw(currentPositionMs);
    };

    const observer = new ResizeObserver(resize);
    observer.observe(container);
    resize();

    return () => observer.disconnect();
  }, [height, draw, currentPositionMs]);

  useEffect(() => {
    draw(currentPositionMs);
  }, [currentPositionMs, peaks, draw]);

  const pixelToMs = useCallback((clientX: number): number => {
    const canvas = canvasRef.current;
    if (!canvas) return 0;
    const rect = canvas.getBoundingClientRect();
    const ratio = (clientX - rect.left) / rect.width;
    const vp = viewportRef.current;
    return vp.start + ratio * (vp.end - vp.start);
  }, []);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const wheelHandler = (e: WheelEvent) => {
      if (!e.ctrlKey) return;
      const zoomFn = onZoomRef.current;
      if (!zoomFn) return;
      e.preventDefault();
      const centerMs = pixelToMs(e.clientX);
      const factor = e.deltaY < 0 ? ZOOM_FACTOR : 1 / ZOOM_FACTOR;
      zoomFn(factor, centerMs);
    };

    canvas.addEventListener("wheel", wheelHandler, { passive: false });
    return () => canvas.removeEventListener("wheel", wheelHandler);
  }, [pixelToMs]);

  function handleMouseDown(e: React.MouseEvent<HTMLCanvasElement>) {
    dragStateRef.current = {
      startX: e.clientX,
      lastX: e.clientX,
      dragged: false,
      viewportStartMs,
    };
  }

  function handleMouseMove(e: React.MouseEvent<HTMLCanvasElement>) {
    const state = dragStateRef.current;
    if (!state) return;
    const delta = e.clientX - state.startX;
    if (!state.dragged && Math.abs(delta) > DRAG_THRESHOLD_PX) {
      state.dragged = true;
    }
    if (state.dragged && onPan) {
      const canvas = canvasRef.current;
      if (!canvas) return;
      const rect = canvas.getBoundingClientRect();
      const span = viewportEndMs - viewportStartMs;
      const panDeltaMs = -((e.clientX - state.lastX) / rect.width) * span;
      onPan(panDeltaMs);
      state.lastX = e.clientX;
    }
  }

  function handleMouseUp(e: React.MouseEvent<HTMLCanvasElement>) {
    const state = dragStateRef.current;
    dragStateRef.current = null;
    if (!state || !state.dragged) {
      const ms = pixelToMs(e.clientX);
      onSeek(Math.max(0, Math.min(durationMs, ms)));
    }
  }

  function handleMouseLeave() {
    dragStateRef.current = null;
  }

  return (
    <div
      ref={containerRef}
      className="relative w-full bg-bg-input rounded-sm overflow-hidden cursor-crosshair select-none flex-1 min-h-0"
    >
      <canvas
        ref={canvasRef}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseLeave}
        className="w-full block h-full"
      />
    </div>
  );
}
