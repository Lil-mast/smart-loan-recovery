import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Borrower workspace",
  robots: { index: false, follow: false },
};

export default function BorrowerLayout({ children }: { children: React.ReactNode }) {
  return children;
}
