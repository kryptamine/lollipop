import { render } from "preact";

import { App } from "./App";
import "./overlay.css";

const root = document.getElementById("root");
if (root === null) {
  throw new Error("index.html has no #root element");
}
render(<App />, root);
