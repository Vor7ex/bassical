import { useCallback, useEffect, useRef, useState } from "react";
import type { TimingPoint, TimeSignature } from "@/lib/types";
import { TapCalibrationPanel } from "./TapCalibrationPanel";

const TIME_SIG_PRESETS: { label: string; value: TimeSignature | null }[] = [
  { label: "4/4", value: null },
  { label: "3/4", value: { numerator: 3, denominator: 4 } },
  { label: "2/4", value: { numerator: 2, denominator: 4 } },
  { label: "6/8", value: { numerator: 6, denominator: 8 } },
  { label: "12/8", value: { numerator: 12, denominator: 8 } },
  { label: "5/4", value: { numerator: 5, denominator: 4 } },
  { label: "7/8", value: { numerator: 7, denominator: 8 } },
];

function timeSigKey(ts: TimeSignature | undefined): string {
  if (!ts) return "4/4";
  return `${ts.numerator}/${ts.denominator}`;
}

function validateBpm(text: string): boolean {
  const v = parseFloat(text);
  return !isNaN(v) && v >= 20 && v <= 400;
}

function validateOffset(text: string, maxMs: number): boolean {
  const v = parseFloat(text);
  return !isNaN(v) && v >= 0 && v <= maxMs;
}

interface TimeSignatureSelectProps {
  value: TimeSignature | undefined;
  onChange: (ts: TimeSignature | undefined) => void;
}

function TimeSignatureSelect({ value, onChange }: TimeSignatureSelectProps) {
  const currentKey = timeSigKey(value);
  const [showCustom, setShowCustom] = useState(
    !TIME_SIG_PRESETS.some((p) => timeSigKey(p.value ?? undefined) === currentKey),
  );

  const handleSelect = useCallback(
    (e: React.ChangeEvent<HTMLSelectElement>) => {
      const val = e.target.value;
      if (val === "custom") {
        setShowCustom(true);
      } else {
        setShowCustom(false);
        const preset = TIME_SIG_PRESETS.find(
          (p) => timeSigKey(p.value ?? undefined) === val,
        );
        onChange(preset?.value ?? undefined);
      }
    },
    [onChange],
  );

  const n = value?.numerator ?? 4;
  const d = value?.denominator ?? 4;

  return (
    <label className="flex flex-col gap-0.5">
      <span className="text-xs text-text-tertiary uppercase tracking-wide">
        Compás
      </span>
      <div className="flex items-center gap-1">
        <select
          value={currentKey}
          onChange={handleSelect}
          className="bg-bg-input border border-border-subtle text-text-primary h-7 px-2 text-caption font-mono flex-1"
        >
          {TIME_SIG_PRESETS.map((p) => (
            <option key={p.label} value={p.label}>
              {p.label}
            </option>
          ))}
          <option value="custom">Custom</option>
        </select>
        {showCustom && (
          <>
            <input
              type="number"
              min={1}
              max={32}
              value={n}
              onChange={(e) =>
                onChange({ numerator: parseInt(e.target.value, 10) || 4, denominator: d })
              }
              className="bg-bg-input border border-border-subtle text-text-primary h-7 px-1.5 text-caption font-mono w-12 text-right"
            />
            <span className="text-text-tertiary text-caption">/</span>
            <select
              value={d}
              onChange={(e) =>
                onChange({ numerator: n, denominator: parseInt(e.target.value, 10) })
              }
              className="bg-bg-input border border-border-subtle text-text-primary h-7 px-1 text-caption font-mono w-14"
            >
              {[1, 2, 4, 8, 16, 32].map((dv) => (
                <option key={dv} value={dv}>
                  {dv}
                </option>
              ))}
            </select>
          </>
        )}
      </div>
    </label>
  );
}

interface CalibrationPanelProps {
  detectedBpm: number | null;
  tapCount: number;
  acceptedCount: number;
  rejectedCount: number;
  canCommit: boolean;
  onTap: () => void;
  onCommit: () => void;
  onCancel: () => void;
  onBeginCalibrating: (index: number) => void;
  calibratingTpIndex: number | null;
}

interface TimingPointPanelProps {
  timingPoints: TimingPoint[];
  selectedIndex: number | null;
  durationMs: number;
  defaultOffsetMs: number;
  onAdd: (offsetMs: number) => void;
  onUpdate: (index: number, patch: Partial<TimingPoint>) => void;
  onRemove: (index: number) => void;
  onSelect: (index: number | null) => void;
  calibrationProps: CalibrationPanelProps;
}

export function TimingPointPanel({
  timingPoints,
  selectedIndex,
  durationMs,
  defaultOffsetMs,
  onAdd,
  onUpdate,
  onRemove,
  onSelect,
  calibrationProps,
}: TimingPointPanelProps) {
  const handleAdd = useCallback(() => {
    onAdd(Math.max(0, Math.min(durationMs, defaultOffsetMs)));
  }, [defaultOffsetMs, durationMs, onAdd]);

  return (
    <div className="w-72 bg-bg-surface border-l border-border-subtle flex flex-col shrink-0">
      <div className="bg-bg-surface border-b border-border-subtle px-3 h-8 flex items-center justify-between shrink-0">
        <span className="text-caption text-text-secondary uppercase tracking-wide">
          Timing Points
        </span>
        <button
          onClick={handleAdd}
          className="text-text-tertiary hover:text-text-primary w-5 h-5 flex items-center justify-center cursor-pointer transition-colors"
          aria-label="Agregar timing point"
        >
          +
        </button>
      </div>

      <div className="flex-1 overflow-y-auto">
        {timingPoints.length === 0 ? (
          <div className="px-3 py-6 text-center">
            <span className="text-caption text-text-tertiary">
              Sin timing points. Haz clic en + para agregar uno.
            </span>
          </div>
        ) : (
          timingPoints.map((tp, idx) => (
            <TimingPointItem
              key={`tp-${idx}`}
              tp={tp}
              index={idx}
              isSelected={selectedIndex === idx}
              isCalibrating={calibrationProps.calibratingTpIndex === idx}
              durationMs={durationMs}
              onSelect={() => onSelect(idx)}
              onUpdate={(patch) => onUpdate(idx, patch)}
              onRemove={() => onRemove(idx)}
              onBeginCalibrating={() => calibrationProps.onBeginCalibrating(idx)}
              calibrationProps={calibrationProps}
            />
          ))
        )}
      </div>
    </div>
  );
}

interface TimingPointItemProps {
  tp: TimingPoint;
  index: number;
  isSelected: boolean;
  isCalibrating: boolean;
  durationMs: number;
  onSelect: () => void;
  onUpdate: (patch: Partial<TimingPoint>) => void;
  onRemove: () => void;
  onBeginCalibrating: () => void;
  calibrationProps: CalibrationPanelProps;
}

function TimingPointItem({
  tp,
  index,
  isSelected,
  isCalibrating,
  durationMs,
  onSelect,
  onUpdate,
  onRemove,
  onBeginCalibrating,
  calibrationProps,
}: TimingPointItemProps) {
  const bpmText = String(tp.bpm);
  const offsetText = String(tp.offsetMs);
  const bpmValid = validateBpm(bpmText);
  const offsetValid = validateOffset(offsetText, durationMs);

  const selectedClasses = isSelected
    ? "border-l-2 border-accent bg-accent-bg"
    : "border-l-2 border-transparent";

  return (
    <div
      onClick={onSelect}
      className={`px-3 py-2 border-b border-border-subtle cursor-pointer ${selectedClasses}`}
    >
      <div className="flex items-center justify-between mb-2">
        <span className="text-caption text-text-tertiary font-mono">
          TP #{index + 1}
        </span>
        <div className="flex items-center gap-1">
          {!isCalibrating && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                onBeginCalibrating();
              }}
              className="text-text-tertiary hover:text-accent w-4 h-4 flex items-center justify-center cursor-pointer transition-colors text-xs leading-none"
              aria-label={`Calibrar TP #${index + 1}`}
              title="Calibrar con tap"
            >
              &#x25C9;
            </button>
          )}
          <button
            onClick={(e) => {
              e.stopPropagation();
              onRemove();
            }}
            className="text-text-tertiary hover:text-text-danger w-4 h-4 flex items-center justify-center cursor-pointer transition-colors text-xs leading-none"
            aria-label={`Eliminar TP #${index + 1}`}
          >
            ×
          </button>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-2">
        <TpField
          label="BPM"
          initialText={bpmText}
          isValid={bpmValid}
          onCommit={(text) => {
            const v = parseFloat(text);
            if (!isNaN(v)) onUpdate({ bpm: v });
          }}
          endSlot={
            <BpmStepper
              value={tp.bpm}
              onChange={(v) => onUpdate({ bpm: v })}
            />
          }
        />
        <TpField
          label="Offset"
          initialText={offsetText}
          isValid={offsetValid}
          onCommit={(text) => {
            const v = parseFloat(text);
            if (!isNaN(v)) onUpdate({ offsetMs: v });
          }}
          endSlot={
            <OffsetStepper
              value={tp.offsetMs}
              maxMs={durationMs}
              onChange={(v) => onUpdate({ offsetMs: v })}
            />
          }
        />
      </div>

      <div className="mt-2">
        <TimeSignatureSelect
          value={tp.timeSignature}
          onChange={(ts) => onUpdate({ timeSignature: ts })}
        />
      </div>

      {isCalibrating && (
        <TapCalibrationPanel
          detectedBpm={calibrationProps.detectedBpm}
          tapCount={calibrationProps.tapCount}
          acceptedCount={calibrationProps.acceptedCount}
          rejectedCount={calibrationProps.rejectedCount}
          canCommit={calibrationProps.canCommit}
          onTap={calibrationProps.onTap}
          onCommit={calibrationProps.onCommit}
          onCancel={calibrationProps.onCancel}
        />
      )}
    </div>
  );
}

function TpField({
  label,
  initialText,
  isValid,
  onCommit,
  endSlot,
}: {
  label: string;
  initialText: string;
  isValid: boolean;
  onCommit: (text: string) => void;
  endSlot?: React.ReactNode;
}) {
  const [text, setText] = useState(initialText);

  useEffect(() => {
    setText(initialText);
  }, [initialText]);

  const handleBlur = useCallback(() => {
    if (isValid) {
      onCommit(text);
    } else {
      setText(initialText);
    }
  }, [isValid, onCommit, initialText, text]);

  return (
    <label className="flex flex-col gap-0.5">
      <span className="text-xs text-text-tertiary uppercase tracking-wide">
        {label}
      </span>
      <div className="flex items-stretch gap-0.5">
        <input
          type="text"
          value={text}
          onChange={(e) => setText(e.target.value)}
          onBlur={handleBlur}
          onKeyDown={(e) => {
            if (e.key === "Enter") handleBlur();
          }}
          className={`bg-bg-input border text-text-primary h-7 px-2 text-caption font-mono text-right flex-1 min-w-0 ${
            isValid ? "border-border-subtle" : "border-danger-border"
          }`}
        />
        {endSlot}
      </div>
    </label>
  );
}

function BpmStepper({ value, onChange }: { value: number; onChange: (v: number) => void }) {
  const apply = (delta: number) => {
    const next = Math.max(20, Math.min(400, +(value + delta).toFixed(2)));
    onChange(next);
  };

  return (
    <div className="flex flex-col w-7 h-7 text-[8px] leading-none shrink-0">
      <button
        onClick={() => apply(0.01)}
        onMouseDown={(e) => e.preventDefault()}
        className="h-[15%] flex items-center justify-center bg-bg-input border border-border-subtle text-text-tertiary hover:text-text-primary hover:border-border-strong cursor-pointer transition-colors"
      >
        +
      </button>
      <button
        onClick={() => apply(0.1)}
        onMouseDown={(e) => e.preventDefault()}
        className="h-[35%] flex items-center justify-center bg-bg-input border border-border-subtle text-text-tertiary hover:text-text-primary hover:border-border-strong cursor-pointer transition-colors border-t-0"
      >
        +
      </button>
      <button
        onClick={() => apply(-0.1)}
        onMouseDown={(e) => e.preventDefault()}
        className="h-[35%] flex items-center justify-center bg-bg-input border border-border-subtle text-text-tertiary hover:text-text-primary hover:border-border-strong cursor-pointer transition-colors border-t-0"
      >
        −
      </button>
      <button
        onClick={() => apply(-0.01)}
        onMouseDown={(e) => e.preventDefault()}
        className="h-[15%] flex items-center justify-center bg-bg-input border border-border-subtle text-text-tertiary hover:text-text-primary hover:border-border-strong cursor-pointer transition-colors border-t-0"
      >
        −
      </button>
    </div>
  );
}

function OffsetStepper({
  value,
  maxMs,
  onChange,
}: {
  value: number;
  maxMs: number;
  onChange: (v: number) => void;
}) {
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const holdTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const signRef = useRef(0);
  const valueRef = useRef(value);
  valueRef.current = value;

  const stopHold = useCallback(() => {
    if (holdTimeoutRef.current) {
      clearTimeout(holdTimeoutRef.current);
      holdTimeoutRef.current = null;
    }
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
  }, []);

  useEffect(() => {
    return () => stopHold();
  }, [stopHold]);

  const step = useCallback(
    (delta: number) => {
      onChange(+(value + delta).toFixed(0));
    },
    [value, onChange],
  );

  const startHold = (sign: 1 | -1) => () => {
    signRef.current = sign;
    step(sign);
    holdTimeoutRef.current = setTimeout(() => {
      intervalRef.current = setInterval(() => {
        const delta = signRef.current;
        const v = +(valueRef.current + delta).toFixed(0);
        if (v >= 0 && v <= maxMs) {
          onChange(v);
        }
      }, 10);
    }, 1000);
  };

  const btn = "h-1/2 flex items-center justify-center bg-bg-input border border-border-subtle text-text-tertiary hover:text-text-primary hover:border-border-strong cursor-pointer transition-colors text-[8px] leading-none";

  return (
    <div className="flex flex-col w-7 h-7 text-[8px] leading-none shrink-0">
      <button
        onMouseDown={(e) => {
          e.preventDefault();
          startHold(1)();
        }}
        onMouseUp={stopHold}
        onMouseLeave={stopHold}
        className={`${btn} rounded-b-none`}
      >
        +
      </button>
      <button
        onMouseDown={(e) => {
          e.preventDefault();
          startHold(-1)();
        }}
        onMouseUp={stopHold}
        onMouseLeave={stopHold}
        className={`${btn} border-t-0 rounded-t-none`}
      >
        −
      </button>
    </div>
  );
}
