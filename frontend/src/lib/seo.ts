import type { Metadata } from "next";
import { faqs } from "@/lib/faqs";
import { getSiteUrl, site } from "@/lib/site";

export function absoluteUrl(path = "/"): string {
  const base = getSiteUrl();
  if (path === "/") {
    return base;
  }
  return `${base}${path.startsWith("/") ? path : `/${path}`}`;
}

export function pageMetadata({
  title,
  description,
  path,
}: {
  title: string;
  description: string;
  path: string;
}): Metadata {
  const url = absoluteUrl(path);
  return {
    title,
    description,
    alternates: { canonical: path },
    openGraph: {
      title: `${title} | ${site.name}`,
      description,
      url,
    },
    twitter: {
      title: `${title} | ${site.name}`,
      description,
    },
  };
}

export function marketingJsonLd(): Record<string, unknown> {
  const url = getSiteUrl();
  const ownerId = `${url}/#person`;
  const orgId = `${url}/#organization`;

  return {
    "@context": "https://schema.org",
    "@graph": [
      {
        "@type": "WebSite",
        "@id": `${url}/#website`,
        url,
        name: site.name,
        description: site.description,
        inLanguage: "en-KE",
        publisher: { "@id": orgId },
      },
      {
        "@type": "Organization",
        "@id": orgId,
        name: site.name,
        url,
        description: site.description,
        founder: { "@id": ownerId },
        sameAs: [site.owner.linktree],
        address: {
          "@type": "PostalAddress",
          addressLocality: site.owner.location,
          addressCountry: "KE",
        },
      },
      {
        "@type": "Person",
        "@id": ownerId,
        name: site.owner.name,
        jobTitle: site.owner.role,
        description: site.owner.bio,
        image: `${url}${site.owner.photo}`,
        url: site.owner.linktree,
        sameAs: [site.owner.linktree],
        address: {
          "@type": "PostalAddress",
          addressLocality: site.owner.location,
          addressCountry: "KE",
        },
        worksFor: { "@id": orgId },
      },
      {
        "@type": "SoftwareApplication",
        name: site.name,
        applicationCategory: "FinanceApplication",
        operatingSystem: "Web",
        description: site.description,
        url,
        offers: {
          "@type": "Offer",
          price: "0",
          priceCurrency: "KES",
        },
        author: { "@id": ownerId },
        provider: { "@id": orgId },
      },
      {
        "@type": "FAQPage",
        mainEntity: faqs.map((item) => ({
          "@type": "Question",
          name: item.q,
          acceptedAnswer: {
            "@type": "Answer",
            text: item.a,
          },
        })),
      },
    ],
  };
}
