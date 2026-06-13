interface StarRatingProps {
  value: number | null;
  onChange: (value: number | null) => void;
}

export function StarRating({ value, onChange }: StarRatingProps) {
  return (
    <div className="flex items-center gap-0.5">
      {[1, 2, 3, 4, 5].map((star) => (
        <button
          key={star}
          type="button"
          onClick={() => onChange(value === star ? null : star)}
          className={`text-sm cursor-pointer transition-colors ${
            star <= (value ?? 0)
              ? "text-accent"
              : "text-text-tertiary hover:text-text-secondary"
          }`}
        >
          ★
        </button>
      ))}
      {value !== null && (
        <button
          type="button"
          onClick={() => onChange(null)}
          className="ml-1 text-caption text-text-tertiary hover:text-text-secondary cursor-pointer"
        >
          ✕
        </button>
      )}
    </div>
  );
}
