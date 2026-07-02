import { escape as _$escape } from "r-server";
import { ssr as _$ssr } from "r-server";
import { ssrHydrationKey as _$ssrHydrationKey } from "r-server";
const template = _$ssr([
	"<div",
	" $ServerOnly><h1>Hello</h1>",
	"",
	"<span>More Text</span></div>"
], _$ssrHydrationKey(), Component({}), _$escape(state.interpolation));
const template2 = Component({ get children() {
	return _$ssr(["<div", " $ServerOnly></div>"], _$ssrHydrationKey());
} });
const template3 = Component({ get children() {
	return [_$ssr(["<div", " $ServerOnly></div>"], _$ssrHydrationKey()), _$ssr(["<span", " $ServerOnly></span>"], _$ssrHydrationKey())];
} });
const template4 = _$ssr(["<div", " $ServerOnly></div>"], _$ssrHydrationKey());
