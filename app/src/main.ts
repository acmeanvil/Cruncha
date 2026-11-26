// SPDX-License-Identifier: GPL-3.0-or-later
import { mount } from "svelte";
import App from "./App.svelte";
import { installPageLogging } from "./lib/log";
import "temml/dist/Temml-Local.css";
import "./app.css";

// The debug log (ADR-0029): uncaught errors, unhandled rejections, the stall watch.
installPageLogging();

export default mount(App, { target: document.getElementById("app")! });
