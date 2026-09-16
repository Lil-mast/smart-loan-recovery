"use client";

import { cn } from "@/lib/utils";

export function AuthPills<T extends string>({
  value,
  onChange,
  options,
}: {
  value: T;
  onChange: (value: T) => void;
  options: { value: T; label: string }[];
}) {
  return (
    <div
      className={cn(
        "grid h-11 w-full rounded-full bg-muted p-1",
        options.length === 3 ? "grid-cols-3" : "grid-cols-2",
      )}
    >
      {options.map((opt) => (
        <button
          key={opt.value}
          type="button"
          onClick={() => onChange(opt.value)}
          className={cn(
            "rounded-full text-xs font-medium sm:text-sm",
            value === opt.value ? "bg-white text-foreground shadow-sm" : "text-muted-foreground",
          )}
        >
          {opt.label}
        </button>
      ))}
    </div>
  );
}
