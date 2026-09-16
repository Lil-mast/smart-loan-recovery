# Marketing UI redesign

Spec for the public site (`/`, `/login`, `/register`). Lender and borrower dashboards keep the existing dark tokens and layout.

Visual references:

- Landing rhythm: [Rescale (Framer)](https://rescale.framer.ai/)
- Auth shell: [`frontend/src/assets/Login _ Sign up 3.png`](../frontend/src/assets/Login%20_%20Sign%20up%203.png)

Owner copy lives in [`frontend/src/lib/site.ts`](../frontend/src/lib/site.ts). Replace the `OWNER_*` placeholders when you have them.

## Isolation

Marketing chrome uses `data-theme="marketing"` on a wrapper in `frontend/src/app/(marketing)/layout.tsx`. Dashboard CSS variables on `:root` in `frontend/src/app/globals.css` stay as they are (`#0f131d` navy, `#adc6ff` primary). Do not restyle `/lender` or `/borrower`.

## Landing (Rescale structure, LendWise copy)

1. Frosted sticky nav: Features, How it works, About, FAQ, Contact, Sign in, Register
2. Hero: atmospheric ice-blue wash, oversized headline, **liquid glass on the display phrase**, dual CTAs
3. Logo / capability marquee
4. Product spotlight (static dashboard preview, not live widgets)
5. Four-step How it works: Register → Track → Flag overdue → Recover
6. Engine band: Rust scoring + Firebase Google (no fake partner logos)
7. About: owner card, contact, short bio
8. FAQ accordion
9. Closing contact CTA + footer

Skip Rescale pricing, journal, YC story, and invented 12K metrics.

Theme: white surfaces, pale sky (`#E8F1FF`), deep blue ink (`#0B3A6A`), one light-blue accent. Outer canvas is a deep ice wash — not the login mock’s burgundy.

## Auth

Shared split-panel shell: left photograph, right white form, rounded outer card on a deep ice background.

Keep existing auth behavior: borrower/lender tabs, Google / email / account ID, lender company register + 4-character ID. Do not add Facebook. Do not fake password reset.

## Sourced components

| Need | Source |
|------|--------|
| shadcn primitives | `pnpm dlx shadcn@latest init -d --base radix` then `button input label checkbox separator accordion tabs card alert sheet` |
| Hero liquid glass text | CSS + SVG displacement in `liquid-glass-text.tsx`, technique from [StarKnightt/liquid-glass](https://github.com/StarKnightt/liquid-glass) (`npx shadcn@latest add https://starknightt.github.io/liquid-glass/r/liquid-glass.json`) and [rdev/liquid-glass-react](https://github.com/rdev/liquid-glass-react) |
| Marquee | Pattern from [Magic UI Marquee](https://magicui.design/docs/components/marquee) |
| Motion | `motion` (Framer Motion). Honor `prefers-reduced-motion`. |
| Icons | `lucide-react` |

## Out of scope

Dashboard restyle, new OAuth providers, real password reset, copying Rescale testimonials or pricing.
