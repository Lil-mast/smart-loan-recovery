"use client";

import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";

const faqs = [
  {
    q: "Is this suitable if my team is new to collections?",
    a: "Yes. Lenders register a company workspace, share a 4-character account ID, and the book shows status, health, and a recommended next step: remind, renegotiate, or escalate.",
  },
  {
    q: "How accurate is the recovery score?",
    a: "The score is deterministic from coverage, days past due, consecutive misses, and schedule lag. It is a rules engine, not a black-box guess, so the same inputs always produce the same band.",
  },
  {
    q: "Can borrowers use Google?",
    a: "Yes. Borrowers join with the lender’s account ID, then Google or email. Lenders sign in with that ID (optional Google later). The same Google account cannot be both roles.",
  },
  {
    q: "What about security?",
    a: "Sessions use an httpOnly cookie. Google sign-in goes through Firebase. Loan and user data stay on your LendWise API — we do not invent demo books.",
  },
  {
    q: "Do you reset passwords by email yet?",
    a: "Not yet. Use Google, your 4-character account ID, or contact the owner listed on this site.",
  },
];

export function FaqSection() {
  return (
    <Accordion type="single" collapsible className="rounded-2xl border border-line bg-card px-4">
      {faqs.map((item, i) => (
        <AccordionItem key={item.q} value={`faq-${i}`}>
          <AccordionTrigger className="text-base">{item.q}</AccordionTrigger>
          <AccordionContent className="text-muted-foreground">{item.a}</AccordionContent>
        </AccordionItem>
      ))}
    </Accordion>
  );
}
