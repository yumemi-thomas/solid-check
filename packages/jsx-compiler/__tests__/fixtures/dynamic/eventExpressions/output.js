import { template as _$template } from "r-dom";
import { addEvent as _$addEvent } from "r-dom";
import { delegateEvents as _$delegateEvents } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<div id=main><button>Change Bound</button><button>Change Bound</button><button>Change Bound</button><button>Change Bound</button><button>Change Bound</button><button>Click Delegated</button><button>Click Delegated</button><button>Click Delegated</button><button>Click Delegated</button><button>Click Delegated`);
function hoisted1() {
	console.log("hoisted");
}
const hoisted2 = () => console.log("hoisted delegated");
const template = (() => {
	var _el$ = _tmpl$();
	var _el$2 = _el$.firstChild;
	var _el$3 = _el$.firstChild.nextSibling;
	var _el$4 = _el$.firstChild.nextSibling.nextSibling;
	var _el$5 = _el$.firstChild.nextSibling.nextSibling.nextSibling;
	var _el$6 = _el$.firstChild.nextSibling.nextSibling.nextSibling.nextSibling;
	var _el$7 = _el$.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling;
	var _el$8 = _el$.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling;
	var _el$9 = _el$.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling;
	var _el$10 = _el$.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling;
	var _el$11 = _el$.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling;
	_el$2.addEventListener("change", () => console.log("bound"));
	_el$3.addEventListener("change", (e) => {
		return ((id) => console.log("bound", id))(id, e);
	});
	_$addEvent(_el$4, "change", handler);
	_el$5.addEventListener("change", handler);
	_$addEvent(_el$6, "change", hoisted1);
	_el$7.$$click = () => console.log("delegated");
	{
		_el$8.$$click = (id) => console.log("delegated", id);
		_el$8.$$clickData = rowId;
	}
	_$addEvent(_el$9, "click", handler, true);
	_el$10.$$click = handler;
	_$addEvent(_el$11, "click", hoisted2, true);
	return _el$;
})();
_$delegateEvents(["click"]);
