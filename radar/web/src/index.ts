import * as ReactDOM from "react-dom/client";
import * as React from "react";
import { initializeAppStore } from "./state";

const container = document.createElement("div");
container.id = "app-container";

const appRoot = ReactDOM.createRoot(container);
document.body.appendChild(container);

import("./ui/app").then(async app => {
    await initializeAppStore();
    appRoot.render(React.createElement(app.App))
});