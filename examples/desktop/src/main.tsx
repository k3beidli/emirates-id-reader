import { createRoot } from "react-dom/client";
import { App } from "./App";
import { Recovery } from "./Recovery";
createRoot(document.getElementById("root")!).render(
  <Recovery>
    <App />
  </Recovery>,
);
