import { cn } from "@/lib/utils";

export function MarqueeRow({
  items,
  className,
}: {
  items: string[];
  className?: string;
}) {
  const loop = [...items, ...items];
  return (
    <div className={cn("overflow-hidden", className)}>
      <div className="marketing-marquee-track flex w-max gap-10 pr-10">
        {loop.map((item, i) => (
          <span
            key={`${item}-${i}`}
            className="font-mono text-[11px] tracking-[0.28em] text-muted-foreground uppercase whitespace-nowrap"
          >
            {item}
          </span>
        ))}
      </div>
    </div>
  );
}
