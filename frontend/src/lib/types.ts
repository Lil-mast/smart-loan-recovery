export type UserRole = "borrower" | "lender";
export type HealthBand = "healthy" | "watch" | "at_risk" | "critical";

export function dashboardPath(role: UserRole): string {
  return role === "lender" ? "/lender" : "/borrower";
}

export type SessionUser = {
  id: string;
  role: UserRole;
  name: string;
};

export type Installment = {
  due: string;
  expected: number;
  covered: number;
  status: string;
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
  paid_amount?: number;
  expected_paid?: number;
  monthly_payment?: number;
  risk_score: number;
  health_score?: number;
  health_band?: string;
  days_past_due?: number;
  days_until_due?: number;
  missed_installments?: number;
  next_due?: string | null;
  coverage_ratio?: number;
  ai_recommendation: string;
  repayment_schedule?: string[];
  installments?: Installment[];
  early_pay_offered?: boolean;
  evaluated_at?: string;
};

export type LoanSignal = {
  id: string;
  loan_id: string;
  borrower_id: string;
  lender_id: string;
  kind: string;
  message: string;
  proposed_date?: string | null;
  created_at: string;
  status: string;
};

export type ApiUser = {
  id: string;
  name: string;
  role: string;
  email?: string | null;
  lender_id?: string | null;
  organization?: string | null;
};
