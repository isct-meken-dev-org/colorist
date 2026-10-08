import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { listen } from "@tauri-apps/api/event";

listen<[string, string]>("rust-log", (event) => {
    const [level, message] = event.payload;

    switch (level) {
        case "error":
            console.error(`[Rust] ${message}`);
            break;
        case "warn":
            console.warn(`[Rust] ${message}`);
            break;
        case "debug":
            console.debug(`[Rust] ${message}`);
            break;
        default:
            console.log(`[Rust] ${message}`);
    }
}).catch(() => undefined);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
        <App />
    </React.StrictMode>,
);
