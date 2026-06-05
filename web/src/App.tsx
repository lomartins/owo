import { Router, Route } from "@solidjs/router";
import { lazy, type JSX } from "solid-js";
import { MonthProvider } from "./lib/useMonth";
import { AppShell } from "./AppShell";
import { ToastHost } from "./components/Toast";

const Dashboard = lazy(() => import("./routes/Dashboard"));
const Transactions = lazy(() => import("./routes/Transactions"));
const AddTransaction = lazy(() => import("./routes/AddTransaction"));
const Budgets = lazy(() => import("./routes/Budgets"));
const Login = lazy(() => import("./routes/Login"));
const Accounts = lazy(() => import("./routes/Accounts"));
const Bills = lazy(() => import("./routes/Bills"));
const Profile = lazy(() => import("./routes/Profile"));
const CategoriesView = lazy(() => import("./routes/Categories"));
const Settings = lazy(() => import("./routes/Settings"));

export default function App(): JSX.Element {
  return (
    <>
      <MonthProvider>
        <Router>
          <Route path="/login" component={Login} />
          <Route path="/" component={AppShell}>
            <Route path="/" component={Dashboard} />
            <Route path="/transactions" component={Transactions} />
            <Route path="/add" component={AddTransaction} />
            <Route path="/budgets" component={Budgets} />
            <Route path="/accounts" component={Accounts} />
            <Route path="/bills" component={Bills} />
            <Route path="/profile" component={Profile} />
            <Route path="/categories" component={CategoriesView} />
            <Route path="/settings" component={Settings} />
          </Route>
          <Route path="*" component={Dashboard} />
        </Router>
      </MonthProvider>
      <ToastHost />
    </>
  );
}
