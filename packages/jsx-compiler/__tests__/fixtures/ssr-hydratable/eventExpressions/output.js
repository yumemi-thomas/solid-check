import { escape as _$escape } from "r-server";
import { ssr as _$ssr } from "r-server";
import { ssrHydrationKey as _$ssrHydrationKey } from "r-server";
function hoistedCustomEvent1() {
	console.log("hoisted");
}
const hoistedcustomevent2 = () => console.log("hoisted");
const template = _$ssr([
	"<div",
	" id=\"main\"><button onchange=\"",
	"\">Change Bound</button><button onChange=\"",
	"\">Change Bound</button><button onclick=\"",
	"\">Click Delegated</button><button onClick=\"",
	"\">Click Delegated</button></div>"
], _$ssrHydrationKey(), _$escape(() => console.log("bound"), true), _$escape([(id) => console.log("bound", id), id], true), _$escape(() => console.log("delegated"), true), _$escape([(id) => console.log("delegated", id), rowId], true));
