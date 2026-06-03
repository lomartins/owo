// Single source of truth for the active month + the user's locale/currency,
// shared across the SPA via Solid context.

import { createContext, createSignal, useContext, type Accessor, type ParentProps, type JSX } from "solid-js";
import { currentMonth } from "./month";
import { currentUser } from "./session";

interface MonthCtx {
  month: Accessor<string>;
  setMonth: (m: string) => void;
  currency: Accessor<string>;
  locale: Accessor<string>;
  refreshToken: Accessor<number>;
  bumpRefresh: () => void;
}

const Ctx = createContext<MonthCtx>();

export function MonthProvider(props: ParentProps): JSX.Element {
  const [month, setMonth] = createSignal<string>(currentMonth());
  const [refreshToken, setRefresh] = createSignal(0);
  const currency = (): string => currentUser()?.default_currency ?? "BRL";
  const locale = (): string => currentUser()?.locale ?? "pt-BR";
  return (
    <Ctx.Provider
      value={{
        month,
        setMonth,
        currency,
        locale,
        refreshToken,
        bumpRefresh: () => setRefresh((n) => n + 1),
      }}
    >
      {props.children}
    </Ctx.Provider>
  );
}

export function useMonth(): MonthCtx {
  const c = useContext(Ctx);
  if (!c) throw new Error("MonthProvider missing");
  return c;
}
