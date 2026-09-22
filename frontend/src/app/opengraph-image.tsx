import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { ImageResponse } from "next/og";
import { site } from "@/lib/site";

export const alt = `${site.name} — AI loan recovery software by ${site.owner.name}`;
export const size = { width: 1200, height: 630 };
export const contentType = "image/png";

export default async function Image() {
  const photo = await readFile(join(process.cwd(), "public/taz.jpeg"));
  const photoSrc = `data:image/jpeg;base64,${photo.toString("base64")}`;

  return new ImageResponse(
    (
      <div
        style={{
          width: "100%",
          height: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: 72,
          background: "linear-gradient(160deg, #E8F1FF 0%, #ffffff 48%, #cfe3ff 100%)",
          color: "#0B3A6A",
          fontFamily: "sans-serif",
        }}
      >
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            justifyContent: "space-between",
            height: "100%",
            maxWidth: 720,
          }}
        >
          <div
            style={{
              display: "flex",
              fontSize: 22,
              letterSpacing: 4,
              textTransform: "uppercase",
              fontWeight: 700,
            }}
          >
            {site.shortName}
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 18 }}>
            <div style={{ display: "flex", fontSize: 58, fontWeight: 800, lineHeight: 1.05 }}>
              {site.name}
            </div>
            <div style={{ display: "flex", fontSize: 26, color: "#3d6fa3" }}>
              AI-driven loan recovery for lenders and borrowers
            </div>
          </div>
          <div style={{ display: "flex", fontSize: 22, color: "#0B3A6A" }}>
            {site.owner.name} · {site.owner.role} · {site.owner.location}
          </div>
        </div>
        <img
          src={photoSrc}
          width={280}
          height={380}
          alt=""
          style={{
            objectFit: "cover",
            objectPosition: "center 12%",
            borderRadius: 28,
          }}
        />
      </div>
    ),
    { ...size },
  );
}
