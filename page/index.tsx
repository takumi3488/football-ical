import { createRoot } from "react-dom/client";
import { Main } from "./main";

const container = document.querySelector("#root");
if (!container) {
	throw new Error("No root element found");
}
const root = createRoot(container);

root.render(<Main />);
