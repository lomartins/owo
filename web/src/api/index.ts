// Typed API methods. Thin wrappers over `request`.

import { request, deviceInfo } from "./client";
import type {
  Account,
  AuthResponse,
  Bill,
  BudgetMonth,
  Category,
  CreateBill,
  CreateBudget,
  CreateTransaction,
  MonthlyReport,
  PayBill,
  UpdateBill,
  PhotoUploadResponse,
  ProfileUpdate,
  Transaction,
  TransactionPage,
  UpdateTransaction,
  User,
} from "./types";

interface ListEnvelope<T> { items: T[]; }

// ---- auth ----
export const auth = {
  login: (email: string, password: string) =>
    request<AuthResponse>("/auth/login", {
      method: "POST",
      body: { email, password, ...deviceInfo() },
    }),
  register: (input: {
    email: string;
    password: string;
    display_name: string;
    default_currency: string;
    locale: string;
  }) =>
    request<AuthResponse>("/auth/register", {
      method: "POST",
      body: { ...input, ...deviceInfo() },
    }),
  me: () => request<{ user: User }>("/auth/me"),
  logout: () => request<void>("/auth/logout", { method: "POST" }),
  updateProfile: (body: ProfileUpdate) =>
    request<{ user: User }>("/auth/me", { method: "PATCH", body }),
  uploadPhoto: (dataUrl: string) =>
    request<PhotoUploadResponse>("/profile/photo", { method: "POST", body: { data_url: dataUrl } }),
};

// ---- accounts ----
export const accounts = {
  list: () => request<ListEnvelope<Account>>("/accounts").then((r) => r.items),
  create: (body: { name: string; type: "asset"; currency: string; initial_balance?: string }) =>
    request<Account>("/accounts", { method: "POST", body }),
};

// ---- categories ----
export const categories = {
  list: () => request<ListEnvelope<Category>>("/categories").then((r) => r.items),
  create: (body: { name: string; kind: "EXPENSE" | "INCOME" | "BOTH" }) =>
    request<Category>("/categories", { method: "POST", body }),
  update: (id: string, body: { name?: string; kind?: string; archived?: boolean; icon?: string | null; color?: string | null }) =>
    request<Category>(`/categories/${id}`, { method: "PATCH", body }),
  remove: (id: string) => request<void>(`/categories/${id}`, { method: "DELETE" }),
};

// ---- transactions ----
export const transactions = {
  list: (q: { month?: string; category_id?: string; account_id?: string; limit?: number }) =>
    request<TransactionPage>("/transactions", { query: q as Record<string, string | number | undefined> }),
  create: (body: CreateTransaction) =>
    request<Transaction>("/transactions", { method: "POST", body }),
  transfer: (body: {
    source_account_id: string;
    destination_account_id: string;
    value: string;
    currency: string;
    tx_date: string;
    description: string;
    payment_method: string;
  }) => request<Transaction>("/transactions/transfer", { method: "POST", body }),
  update: (id: string, body: UpdateTransaction) =>
    request<Transaction>(`/transactions/${id}`, { method: "PATCH", body }),
  remove: (id: string) => request<void>(`/transactions/${id}`, { method: "DELETE" }),
};

// ---- budgets ----
export const budgets = {
  list: (month: string) => request<BudgetMonth>("/budgets", { query: { month } }),
  upsert: (body: CreateBudget) => request<unknown>("/budgets", { method: "POST", body }),
  update: (id: string, estimated_amount: string) =>
    request<unknown>(`/budgets/${id}`, { method: "PATCH", body: { estimated_amount } }),
  remove: (id: string) => request<void>(`/budgets/${id}`, { method: "DELETE" }),
};

// ---- reports ----
export const reports = {
  monthly: (month: string) => request<MonthlyReport>("/reports/monthly", { query: { month } }),
  netWorth: (opts?: { to?: string; months?: number }) =>
    request<import("./types").NetWorthReport>("/reports/net-worth", { query: opts }),
};

// ---- bills (Despesas fixas) ----
export const bills = {
  list: (month?: string) =>
    request<ListEnvelope<Bill>>("/bills", { query: month ? { month } : undefined }).then((r) => r.items),
  create: (body: CreateBill) => request<Bill>("/bills", { method: "POST", body }),
  update: (id: string, body: UpdateBill) =>
    request<Bill>(`/bills/${id}`, { method: "PATCH", body }),
  remove: (id: string) => request<void>(`/bills/${id}`, { method: "DELETE" }),
  pay: (id: string, month: string, body: PayBill) =>
    request<Transaction>(`/bills/${id}/pay`, { method: "POST", query: { month }, body }),
  reset: (id: string, month: string) =>
    request<Bill>(`/bills/${id}/reset`, { method: "POST", query: { month } }),
};
