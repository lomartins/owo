#!/usr/bin/env node
/**
 * Import June 2026 data for Luisa from the spreadsheet images.
 * Idempotent: safe to re-run. Creates accounts/categories if missing,
 * upserts budgets, creates bills (skipping by description match), posts income.
 *
 * Usage:
 *   OWO_TOKEN=... OWO_BASE=http://127.0.0.1:8080 node scripts/import_june_2026.mjs
 */

const BASE = process.env.OWO_BASE ?? "http://127.0.0.1:8080";
const TOKEN = process.env.OWO_TOKEN;
if (!TOKEN) {
  console.error("OWO_TOKEN env var required (bearer token from localStorage 'owo.token')");
  process.exit(1);
}

const MONTH = "2026-06";
const CURRENCY = "BRL";

async function api(method, path, body) {
  const res = await fetch(`${BASE}/api/v1${path}`, {
    method,
    headers: {
      "Authorization": `Bearer ${TOKEN}`,
      ...(body !== undefined ? { "Content-Type": "application/json" } : {}),
    },
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
  if (res.status === 204) return null;
  const text = await res.text();
  const data = text ? safeJSON(text) : null;
  if (!res.ok) {
    const err = new Error(`${method} ${path} → ${res.status} ${text}`);
    err.status = res.status;
    err.body = data;
    throw err;
  }
  return data;
}
function safeJSON(s) { try { return JSON.parse(s); } catch { return s; } }

function log(...args) { console.log("·", ...args); }
function warn(...args) { console.warn("!", ...args); }

// ---------------------------------------------------------------------------
// 1. Bootstrap accounts: Nubank (asset), Poupança (asset).
//    Carteira auto-provisioned at register.
// ---------------------------------------------------------------------------
async function ensureAccount(name, type, initial = "0.00") {
  const existing = (await api("GET", "/accounts")).items.find((a) => a.name === name);
  if (existing) {
    log(`account ${name} exists (${existing.id.slice(0, 8)})`);
    return existing;
  }
  const a = await api("POST", "/accounts", {
    name, type, currency: CURRENCY, initial_balance: initial,
  });
  log(`account ${name} created (${a.id.slice(0, 8)})`);
  return a;
}

async function findCategory(name) {
  const all = (await api("GET", "/categories")).items;
  return all.find((c) => c.name === name);
}

// ---------------------------------------------------------------------------
// 2. Budgets for June (one PUT/POST per category).
// ---------------------------------------------------------------------------
const BUDGETS = [
  ["Food",          "100.00"],
  ["Leisure",       "300.00"],
  ["Transport",     "300.00"],
  ["Health",        "150.00"],
  ["Education",     "0.00"],
  ["Clothes",       "0.00"],
  ["Home",          "100.00"],
  ["Pet",           "0.00"],
  ["Subscriptions", "119.70"],
  ["Other",       "2360.00"],
];

async function setBudgets() {
  for (const [seedName, amount] of BUDGETS) {
    const cat = await findCategory(seedName);
    if (!cat) { warn(`category ${seedName} missing`); continue; }
    await api("POST", "/budgets", {
      category_id: cat.id,
      month: MONTH,
      estimated_amount: amount,
      currency: CURRENCY,
    });
    log(`budget ${seedName}: R$ ${amount}`);
  }
}

// ---------------------------------------------------------------------------
// 3. Recurring bills (Despesas fixas). Category mapping per user defaults.
// ---------------------------------------------------------------------------
const BILLS = [
  // description,  due_day, value,     payment_method, category_seed
  ["Aluguel",        5, "2950.66", "PIX",    "Home"],
  ["Condomínio",    10,  "400.00", "CASH",   "Home"],
  ["Energia",       15,  "500.00", "BOLETO", "Home"],
  // Internet skipped — spreadsheet had no value.
  ["Psicólogo",     20,  "600.00", "PIX",    "Health"],
  ["Academia",       5,   "89.90", "CARD",   "Health"],
  ["Mercado",       25, "1200.00", "VA",     "Food"],
  ["Dívida",        10,  "600.00", "PIX",    "Other"],
  // "Guardar" is handled separately as a monthly transfer to Poupança.
];

async function ensureBill(defaults, defaultAccountId) {
  const [desc, due_day, value, payment_method, catSeed] = defaults;
  const existing = (await api("GET", `/bills?month=${MONTH}`)).items
    .find((b) => b.description === desc);
  if (existing) { log(`bill ${desc} exists (${existing.id.slice(0, 8)})`); return existing; }
  const cat = await findCategory(catSeed);
  const b = await api("POST", "/bills", {
    description: desc,
    value,
    currency: CURRENCY,
    due_day,
    account_id: defaultAccountId,
    category_id: cat ? cat.id : null,
  });
  log(`bill ${desc}: R$ ${value} (dia ${due_day}, ${payment_method})`);
  return b;
}

// ---------------------------------------------------------------------------
// 4. June income (Salário, VA, Auxílio).
//    Posted as transactions with destination = default asset, category derived.
// ---------------------------------------------------------------------------
const INCOMES = [
  // description,         value,         payment_method, category_seed
  ["Salário",          "10342.36", "PIX", "Other"],
  ["VA",                "1200.00", "VA",  "Food"],
  ["Auxílio Home Office", "100.00", "PIX", "Other"],
];

async function postIncome(defaultAccountId) {
  for (const [desc, value, pm, catSeed] of INCOMES) {
    const cat = await findCategory(catSeed);
    if (!cat) { warn(`category ${catSeed} missing for ${desc}`); continue; }
    await api("POST", "/transactions", {
      destination_account_id: defaultAccountId,
      category_id: cat.id,
      payment_method: pm,
      value,
      currency: CURRENCY,
      description: desc,
      tx_date: `${MONTH}-01`,
      paid: true,
    });
    log(`income ${desc}: R$ ${value}`);
  }
}

// ---------------------------------------------------------------------------
// 5. Guardar = monthly transfer Nubank → Poupança.
// ---------------------------------------------------------------------------
async function guardar(nubankId, poupancaId) {
  await api("POST", "/transactions/transfer", {
    source_account_id: nubankId,
    destination_account_id: poupancaId,
    value: "1000.00",
    currency: CURRENCY,
    tx_date: `${MONTH}-05`,
    description: "Guardar",
    payment_method: "PIX",
  });
  log(`transfer Guardar: R$ 1.000,00 → Poupança`);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------
(async () => {
  const me = await api("GET", "/auth/me");
  console.log(`Importing for ${me.user.email} → month ${MONTH}\n`);

  const nubank   = await ensureAccount("Nubank",   "asset");
  const poupanca = await ensureAccount("Poupança", "asset");
  // Default asset for income + bills.
  const defaultAsset = nubank.id;

  console.log("\n— Budgets —");
  await setBudgets();

  console.log("\n— Bills —");
  for (const row of BILLS) await ensureBill(row, defaultAsset);

  console.log("\n— Income —");
  await postIncome(defaultAsset);

  console.log("\n— Guardar (savings transfer) —");
  await guardar(nubank.id, poupanca.id);

  console.log("\nDone.");
})().catch((e) => {
  console.error("Import failed:", e.message);
  if (e.body) console.error(e.body);
  process.exit(1);
});
