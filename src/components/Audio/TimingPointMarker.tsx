import type { TimingPoint } from "@/lib/types";

interface TimingPointMarkerProps {
  tp: TimingPoint;
  index: number;
  isSelected: boolean;
  viewportStartMs: number;
  viewportEndMs: number;
  onSelect: (index: number) => void;
}

export function TimingPointMarker({
  tp,
  index,
  isSelected,
  viewportStartMs,
  viewportEndMs,
  onSelect,
}: TimingPointMarkerProps) {
  const span = viewportEndMs - viewportStartMs;
  const ratio = span > 0 ? (tp.offsetMs - viewportStartMs) / span : 0;
  const visible =
    span > 0 && tp.offsetMs >= viewportStartMs && tp.offsetMs <= viewportEndMs;

  if (!visible) return null;

  const lineColor = isSelected ? "oklch(0.72 0.16 85)" : "oklch(0.65 0.18 25)";
  const labelBg = isSelected
    ? "oklch(0.72 0.16 85)"
    : "oklch(0.65 0.18 25 / 0.85)";
  const labelText = isSelected ? "text-bg-root" : "text-white";

  return (
    <div
      onClick={(e) => {
        e.stopPropagation();
        onSelect(index);
      }}
      className="absolute inset-y-0 z-10 cursor-pointer"
      style={{ left: `${ratio * 100}%` }}
    >
      <div
        className="absolute inset-y-0"
        style={{
          background: lineColor,
          width: isSelected ? 4 : 3,
        }}
      />
      <div
        className="absolute top-2 left-1 whitespace-nowrap leading-tight"
        style={{
          background: labelBg,
          color: labelText,
          padding: "3px 8px",
          fontSize: 11,
          fontFamily: '"Consolas", "Courier New", monospace',
        }}
      >
        <span className="font-bold block text-sm">
          {tp.bpm.toFixed(1)} bpm
        </span>
        <span className="opacity-80 text-xs">
          {tp.offsetMs.toFixed(0)} ms
        </span>
      </div>
    </div>
  );
}
