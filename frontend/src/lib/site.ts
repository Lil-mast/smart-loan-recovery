/** Public marketing copy, contact, and SEO source of truth. */
export const site = {
  name: "LendWise Recovery",
  shortName: "LendWise",
  tagline: "AI-driven loan recovery for the modern lender",
  description:
    "LendWise Recovery is AI-driven loan recovery software for lenders and borrowers. Track loans, flag overdues, and get the next action — remind, renegotiate, or escalate. Built in Nairobi by Christian Tazma.",
  keywords: [
    "LendWise Recovery",
    "LendWise",
    "AI loan recovery",
    "loan recovery software",
    "debt recovery Kenya",
    "collections software Nairobi",
    "Christian Tazma",
    "lender workspace",
    "borrower dashboard",
  ],
  owner: {
    name: "Christian Tazma",
    role: "Software Developer & AI Specialist",
    location: "Nairobi",
    email: "",
    phone: "",
    linktree: "https://linktr.ee/Christian_Tazma",
    bio: "Christian is a creative and forward-thinking Software Developer based in Nairobi. He specializes in building powerful systems that harness AI to tackle real-world challenges.",
    photo: "/taz.jpeg",
  },
} as const;

function stripSlash(url: string): string {
  return url.replace(/\/$/, "");
}

/** Canonical origin. Set NEXT_PUBLIC_SITE_URL in production so search engines get the right host. */
export function getSiteUrl(): string {
  const fromEnv = process.env.NEXT_PUBLIC_SITE_URL;
  if (fromEnv) {
    return stripSlash(fromEnv);
  }

  const vercelProd = process.env.VERCEL_PROJECT_PRODUCTION_URL;
  if (vercelProd) {
    return stripSlash(vercelProd.startsWith("http") ? vercelProd : `https://${vercelProd}`);
  }

  if (process.env.VERCEL_URL) {
    return `https://${process.env.VERCEL_URL.replace(/^https?:\/\//, "")}`;
  }

  return "http://localhost:3001";
}
