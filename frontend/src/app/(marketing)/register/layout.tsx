import type { Metadata } from "next";
import { pageMetadata } from "@/lib/seo";

export const metadata: Metadata = pageMetadata({
  title: "Create a workspace",
  description:
    "Register a LendWise Recovery lender workspace or join as a borrower with a company account ID. Built in Nairobi by Christian Tazma.",
  path: "/register",
});

export default function RegisterLayout({ children }: { children: React.ReactNode }) {
  return children;
}
