"use client";

import { useEffect } from "react";

export function MarketingTheme() {
  useEffect(() => {
    const root = document.documentElement;
    root.setAttribute("data-theme", "marketing");
    return () => {
      root.removeAttribute("data-theme");
    };
  }, []);
  return null;
}
