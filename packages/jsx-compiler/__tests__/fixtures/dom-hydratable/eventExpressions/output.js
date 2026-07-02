import { template as _$template } from "r-dom";
import { getNextElement as _$getNextElement } from "r-dom";
import { delegateEvents as _$delegateEvents } from "r-dom";
import { runHydrationEvents as _$runHydrationEvents } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<div id=main><button>Change Bound</button><button>Change Bound</button><button>Click Delegated</button><button>Click Delegated`);
function hoistedCustomEvent1() {
	console.log("hoisted");
}
const hoistedcustomevent2 = () => console.log("hoisted");
const template = (() => {
	var _el$ = _$getNextElement(_tmpl$);
	var _el$2 = _el$.firstChild;
	_el$2.addEventListener("change", () => console.log("bound"));
	var _el$3 = _el$.firstChild.nextSibling;
	_el$3.addEventListener("change", (e) => {
		return ((id) => console.log("bound", id))(id, e);
	});
	var _el$4 = _el$.firstChild.nextSibling.nextSibling;
	{
		_el$4.$$click = () => console.log("delegated");
		_$runHydrationEvents();
	}
	var _el$5 = _el$.firstChild.nextSibling.nextSibling.nextSibling;
	{
		{
			_el$5.$$click = (id) => console.log("delegated", id);
			_el$5.$$clickData = rowId;
		}
		_$runHydrationEvents();
	}
	return _el$;
})();
_$delegateEvents(["click"]);
