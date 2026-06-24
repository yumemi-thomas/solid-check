import { escape as _$escape } from "r-server";
import { ssr as _$ssr } from "r-server";
// Duplicate attributes on the same element resolve to the last value
// (matching JSX spread semantics: later attributes override earlier ones).
// This test keeps the `class=` case specifically since it used to be a
// special compiler path.
const dynamicClass = () => "dyn";
const flag = true;
const t1 = _$ssr("<div class=\"a\" class=\"b\">static static</div>");
const t2 = _$ssr(["<div class=\"a\" class=\"", "\">static + dynamic</div>"], _$escape(dynamicClass(), true));
const t3 = _$ssr([
	"<div class=\"",
	"\" class=\"",
	"\">two dynamic</div>"
], _$escape(dynamicClass(), true), _$escape(flag ? "on" : "off", true));
const t4 = _$ssr(["<div class=\"base\" class=\"", "\">string + object</div>"], _$escape({
	active: flag,
	dim: !flag
}, true));
const t5 = _$ssr("<div class=\"a\" class=\"b\" class=\"c\">three statics</div>");
