export type UserRole = "borrower" | "lender";

export type SessionUser = {
  id: string;
  role: UserRole;
  name: string;
};

export type Loan = {
  id: string;
  borrower_id: string;
  lender_id: string;
  principal: number;
  amount: number;
  interest_rate: number;
  status: string;
  recovery_status: number;
  outstanding_amount: number;
  risk_score: number;
  ai_recommendation: string;
};

export type ApiUser = {
  id: string;
  name: string;
  role: string;
  email?: string | null;
  lender_id?: string | null;
  organization?: string | null;
};
