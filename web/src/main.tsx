import { render } from "solid-js/web";
import App from "./App";
import { I18nProvider } from "./lib/i18n";
import "./styles/app.css";

const root = document.getElementById("app");
if (!root) throw new Error("#app missing");
render(
  () => (
    <I18nProvider>
      <App />
    </I18nProvider>
  ),
  root,
);
