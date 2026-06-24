import { escape as _$escape } from "r-server";
import { ssr as _$ssr } from "r-server";
import { ssrHydrationKey as _$ssrHydrationKey } from "r-server";
const multiStatic = [_$ssr(["<div", ">First</div>"], _$ssrHydrationKey()), _$ssr(["<div", ">Last</div>"], _$ssrHydrationKey())];
const multiExpression = [
	_$ssr(["<div", ">First</div>"], _$ssrHydrationKey()),
	inserted,
	_$ssr(["<div", ">Last</div>"], _$ssrHydrationKey()),
	"After"
];
const multiDynamic = [
	_$ssr([
		"<div",
		" id=\"",
		"\">First</div>"
	], _$ssrHydrationKey(), _$escape(state.first, true)),
	state.inserted,
	_$ssr([
		"<div",
		" id=\"",
		"\">Last</div>"
	], _$ssrHydrationKey(), _$escape(state.last, true)),
	"After"
];
const singleExpression = inserted;
const singleDynamic = inserted();
const firstStatic = [inserted, _$ssr(["<div", "></div>"], _$ssrHydrationKey())];
const firstDynamic = [inserted(), _$ssr(["<div", "></div>"], _$ssrHydrationKey())];
const firstComponent = [Component({}), _$ssr(["<div", "></div>"], _$ssrHydrationKey())];
const lastStatic = [_$ssr(["<div", "></div>"], _$ssrHydrationKey()), inserted];
const lastDynamic = [_$ssr(["<div", "></div>"], _$ssrHydrationKey()), inserted()];
const lastComponent = [_$ssr(["<div", "></div>"], _$ssrHydrationKey()), Component({})];
const spaces = [
	_$ssr(["<span", ">1</span>"], _$ssrHydrationKey()),
	" ",
	_$ssr(["<span", ">2</span>"], _$ssrHydrationKey()),
	" ",
	_$ssr(["<span", ">3</span>"], _$ssrHydrationKey())
];
const multiLineTrailing = [
	_$ssr(["<span", ">1</span>"], _$ssrHydrationKey()),
	_$ssr(["<span", ">2</span>"], _$ssrHydrationKey()),
	_$ssr(["<span", ">3</span>"], _$ssrHydrationKey())
];
