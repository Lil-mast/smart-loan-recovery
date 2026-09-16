import { MarketingTheme } from "@/components/marketing/marketing-theme";

export default function MarketingLayout({ children }: { children: React.ReactNode }) {
  return (
    <div data-theme="marketing" className="min-h-full bg-background text-foreground">
      <MarketingTheme />
      {children}
    </div>
  );
}
