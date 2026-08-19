import { mount } from "svelte";
import "./styles/fonts.css";
import "./styles/tokens.css";
import "./styles/base.css";
import App from "./routes/main/App.svelte";

export default mount(App, { target: document.getElementById("app")! });
