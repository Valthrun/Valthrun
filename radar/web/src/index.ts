import * as ReactDOM from "react-dom/client";
import * as React from "react";

const container = document.createElement("div");
container.id = "app-container";

const appRoot = ReactDOM.createRoot(container);
document.body.appendChild(container);

import("./ui/app").then(app => {
    appRoot.render(React.createElement(app.App))
});