import * as ReactDOM from "react-dom/client";
import { App } from "./ui/app";
import * as React from "react";

const container = document.createElement("div");
container.id = "app-container";

const appRoot = ReactDOM.createRoot(container);
appRoot.render(
    React.createElement(App)
);
document.body.appendChild(container);