// Wire shapes (mirrors backend domain). Money fields are JSON decimal strings.

export type Decimal = string;

export type AccountType = "asset" | "credit_card" | "liability" | "revenue" | "expense";

export interface Account {
  id: string;
  name: string;
  type: AccountType;
  currency: string;
  initial_balance: Decimal;
  current_balance: Decimal;
  archived: boolean;
  created_at: string;
  updated_at: string;
}

export interface Category {
  id: string;
  name: string;
  parent_id: string | null;
  icon: string | null;
  color: string | null;
  kind: "INCOME" | "EXPENSE" | "BOTH";
  archived: boolean;
  created_at: string;
  updated_at: string;
}

export type PaymentMethod = "PIX" | "CASH" | "BOLETO" | "VA" | "DEBIT" | "CREDIT" | "TED";

export type TransactionKind = "deposit" | "withdrawal" | "transfer";

export interface Transaction {
  id: string;
  source_account_id: string;
  destination_account_id: string;
  category_id: string | null;
  payment_method: PaymentMethod | string;
  value: Decimal;
  currency: string;
  fx_rate: string | null;
  description: string;
  tx_date: string;
  paid: boolean;
  receipt_url: string | null;
  picture_url: string | null;
  card_id: string | null;
  bill_id: string | null;
  invoice_id: string | null;
  created_at: string;
  updated_at: string;
  kind: TransactionKind;
}

export interface UpdateTransaction {
  source_account_id?: string;
  destination_account_id?: string;
  category_id?: string | null;
  payment_method?: PaymentMethod | string;
  value?: Decimal;
  description?: string;
  tx_date?: string;
  paid?: boolean;
}

export interface CreateTransaction {
  /** Omit one leg + provide category_id to let the backend derive the bucket. */
  source_account_id?: string;
  destination_account_id?: string;
  category_id?: string | null;
  payment_method: PaymentMethod | string;
  value: Decimal;
  currency: string;
  description: string;
  tx_date: string;
  paid?: boolean;
}

export interface Bill {
  id: string;
  account_id: string | null;
  category_id: string | null;
  description: string;
  value: Decimal;
  currency: string;
  due_day: number;
  /** Per-month payment flag (server computes via bill_payments for the queried month). */
  paid: boolean;
  paid_at: string | null;
  paid_transaction_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateBill {
  description: string;
  value: Decimal;
  currency: string;
  due_day: number;
  account_id?: string | null;
  category_id?: string | null;
}

export type BillEditScope = "this_month" | "this_and_next" | "all";

export interface UpdateBill {
  description?: string;
  value?: Decimal;
  due_day?: number;
  account_id?: string | null;
  category_id?: string | null;
  /** Default 'all' (legacy patch-the-template behavior). */
  scope?: BillEditScope;
  /** Reference month (YYYY-MM) for scoped edits. Defaults to current. */
  month?: string;
}

export interface PayBill {
  source_account_id: string;
  tx_date?: string;
  payment_method?: string;
  /** Override for this month's amount. */
  amount?: Decimal;
}

export interface BudgetRow {
  category_id: string;
  category_name: string;
  icon: string | null;
  color: string | null;
  budget_id: string | null;
  estimated: Decimal;
  spent: Decimal;
  difference: Decimal;
}

export interface BudgetMonth {
  month: string;
  items: BudgetRow[];
}

export interface CreateBudget {
  category_id: string;
  month: string;
  estimated_amount: Decimal;
  currency: string;
}

export interface MonthlyReport {
  month: string;
  income_total: Decimal;
  spent_total: Decimal;
  budget_balance: Decimal;
  carry_over_in: Decimal;
  carry_over_out: Decimal;
}

export interface NetWorthPoint {
  month: string;
  net_worth: Decimal;
}

export interface NetWorthReport {
  points: NetWorthPoint[];
  current: Decimal;
  change_pct: string | null;
}

export interface User {
  id: string;
  email: string;
  display_name: string;
  default_currency: string;
  locale: string;
  is_admin: boolean;
  partner_name: string | null;
  photo_url: string | null;
  created_at: string;
  updated_at: string;
}

export interface ProfileUpdate {
  display_name?: string;
  default_currency?: string;
  locale?: string;
  partner_name?: string | null;
}

export interface PhotoUploadResponse {
  photo_url: string;
}

export interface SessionToken {
  id: string;
  token: string;
  expires_at: string;
}

export interface AuthResponse {
  user: User;
  session: SessionToken;
}

export interface TransactionPage {
  items: Transaction[];
  page: {
    limit: number;
    order: string;
    returned: number;
    has_more: boolean;
    next_cursor?: string;
  };
}

export interface ApiErrorBody {
  error: {
    code: string;
    message: string;
    details?: unknown;
  };
}
