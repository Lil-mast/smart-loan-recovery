export function LiquidGlassText({ children }: { children: string }) {
  return (
    <>
      <svg width="0" height="0" className="absolute" aria-hidden>
        <filter id="liquid-glass-filter">
          <feTurbulence
            type="fractalNoise"
            baseFrequency="0.012 0.02"
            numOctaves="2"
            seed="3"
            result="noise"
          />
          <feDisplacementMap
            in="SourceGraphic"
            in2="noise"
            scale="3"
            xChannelSelector="R"
            yChannelSelector="G"
          />
        </filter>
      </svg>
      <span className="liquid-glass-text">{children}</span>
    </>
  );
}
