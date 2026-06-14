import { useState, useRef } from "react";

interface TagInputProps {
  tags: string[];
  onChange: (tags: string[]) => void;
}

export function TagInput({ tags, onChange }: TagInputProps) {
  const [input, setInput] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  function addTag() {
    const tag = input.trim().toLowerCase();
    if (tag && !tags.includes(tag)) {
      onChange([...tags, tag]);
    }
    setInput("");
  }

  function removeTag(tag: string) {
    onChange(tags.filter((t) => t !== tag));
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      addTag();
    }
    const canRemoveLast = e.key === "Backspace" && input === "" && tags.length > 0;
    if (canRemoveLast) {
      removeTag(tags[tags.length - 1]);
    }
  }

  return (
    <div
      className="flex flex-wrap items-center gap-1.5 bg-bg-input border border-border-subtle rounded-sm px-2 py-1.5 min-h-[32px] cursor-text"
      onClick={() => inputRef.current?.focus()}
    >
      {tags.map((tag) => (
        <span
          key={tag}
          className="inline-flex items-center gap-1 bg-accent-bg text-accent text-caption px-2 py-0.5 rounded-sm"
        >
          {tag}
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              removeTag(tag);
            }}
            className="text-accent-muted hover:text-accent cursor-pointer text-[10px]"
          >
            ✕
          </button>
        </span>
      ))}
      <input
        ref={inputRef}
        type="text"
        value={input}
        onChange={(e) => setInput(e.target.value)}
        onKeyDown={handleKeyDown}
        onBlur={addTag}
        placeholder={tags.length === 0 ? "Escribir y Enter..." : ""}
        className="flex-1 min-w-[80px] bg-transparent text-text-primary text-body outline-none placeholder:text-text-tertiary"
      />
    </div>
  );
}
