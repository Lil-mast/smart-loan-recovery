export function BrandMark({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 32 24" aria-hidden className={className}>
      <path
        d="M10 12c0-4.4 3.1-8 7-8s7 3.6 7 8-3.1 8-7 8"
        fill="none"
        stroke="currentColor"
        strokeWidth="2.2"
        strokeLinecap="round"
      />
      <path
        d="M22 12c0 4.4-3.1 8-7 8s-7-3.6-7-8 3.1-8 7-8"
        fill="none"
        stroke="currentColor"
        strokeWidth="2.2"
        strokeLinecap="round"
      />
    </svg>
  );
}
