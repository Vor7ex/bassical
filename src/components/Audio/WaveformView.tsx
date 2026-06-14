import { useEffect, useRef, useCallback } from "react";

interface WaveformViewProps {
  peaks: number[];
  currentPositionMs: number;
  durationMs: number;
  onSeek: (positionMs: number) => void;
  height?: number;
}

interface CanvasCtx {
  ctx: CanvasRenderingContext2D;
  width: number;
  height: number;
  dpr: number;
}

function drawBars(c: CanvasCtx, peaks: number[]) {
  const h = c.height * c.dpr;
  const barWidth = 2 * c.dpr;
  const gap = 1 * c.dpr;
  const totalBarWidth = barWidth + gap;
  const numBars = Math.floor(c.width / totalBarWidth);
  const centerY = h / 2;

  c.ctx.fillStyle = "oklch(0.45 0.10 155)";
  for (let i = 0; i < numBars && i < peaks.length; i++) {
    const peakIdx = Math.floor((i / numBars) * peaks.length);
    const amplitude = peaks[peakIdx] * (h * 0.45);
    const x = i * totalBarWidth;
    c.ctx.fillRect(x, centerY - amplitude, barWidth, amplitude);
    c.ctx.fillRect(x, centerY, barWidth, amplitude);
  }
}

function drawPlayhead(c: CanvasCtx, positionMs: number, durationMs: number) {
  const h = c.height * c.dpr;
  const playheadX = durationMs > 0 ? (positionMs / durationMs) * c.width : 0;
  c.ctx.fillStyle = "oklch(0.72 0.18 155)";
  c.ctx.fillRect(playheadX - 1 * c.dpr, 0, 2 * c.dpr, h);
}

function renderFrame(
  canvas: HTMLCanvasElement,
  peaks: number[],
  positionMs: number,
  durationMs: number,
  height: number,
) {
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  const dpr = window.devicePixelRatio || 1;
  const c: CanvasCtx = { ctx, width: canvas.width, height, dpr };
  ctx.clearRect(0, 0, c.width, c.height);
  if (peaks.length === 0) return;
  drawBars(c, peaks);
  drawPlayhead(c, positionMs, durationMs);
}

export function WaveformView({
  peaks,
  currentPositionMs,
  durationMs,
  onSeek,
  height = 200,
}: WaveformViewProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const peaksRef = useRef(peaks);
  peaksRef.current = peaks;

  const draw = useCallback(
    (posMs: number) => {
      const canvas = canvasRef.current;
      if (canvas) renderFrame(canvas, peaksRef.current, posMs, durationMs, height);
    },
    [height, durationMs],
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

  function handleClick(e: React.MouseEvent<HTMLCanvasElement>) {
    const canvas = canvasRef.current;
    if (!canvas || durationMs <= 0) return;
    const rect = canvas.getBoundingClientRect();
    onSeek(((e.clientX - rect.left) / rect.width) * durationMs);
  }

  return (
    <div
      ref={containerRef}
      className="relative w-full bg-bg-input rounded-sm overflow-hidden cursor-crosshair"
    >
      <canvas ref={canvasRef} onClick={handleClick} className="w-full block" />
    </div>
  );
}
