import { scope as _$scope } from "r-server";
import { escape as _$escape } from "r-server";
import { ssr as _$ssr } from "r-server";
import { ssrHydrationKey as _$ssrHydrationKey } from "r-server";
const template1 = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$escape(simple));
const template2 = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$escape(state.dynamic));
const template3 = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$escape(simple ? good : bad));
const template4 = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$scope(() => {
	return _$escape(simple ? good() : bad);
}));
const template4a = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$escape(simple ? good.good : bad));
const template5 = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$scope(() => {
	return _$escape(state.dynamic ? good() : bad);
}));
const template5a = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$escape(state.dynamic ? good.good : bad));
const template6 = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$scope(() => {
	return _$escape(state.dynamic && good());
}));
const template6a = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$escape(state.dynamic && good.good));
const template7 = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$escape(state.count > 5 ? state.dynamic ? best : good() : bad));
const template7a = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$escape(state.count > 5 ? state.dynamic ? best : good.good : bad));
const template8 = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$scope(() => {
	return _$escape(state.dynamic && state.something && good());
}));
const template8a = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$escape(state.dynamic && state.something && good.good));
const template9 = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$escape(state.dynamic && good() || bad));
const template9a = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$escape(state.dynamic && good.good || bad));
const template10 = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$escape(state.a ? "a" : state.b ? "b" : state.c ? "c" : "fallback"));
const template11 = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$scope(() => {
	return _$escape(state.a ? a() : state.b ? b() : state.c ? "c" : "fallback");
}));
const template11a = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$escape(state.a ? a.a : state.b ? b.b : state.c ? "c" : "fallback"));
const template12 = Comp({ render: state.dynamic ? good() : bad });
const template12a = Comp({ render: state.dynamic ? good.goood : bad });
// no dynamic predicate
const template13 = Comp({ render: state.dynamic ? good : bad });
const template14 = Comp({ render: state.dynamic && good() });
const template14a = Comp({ render: state.dynamic && good.good });
// no dynamic predicate
const template15 = Comp({ render: state.dynamic && good });
const template16 = Comp({ render: state.dynamic || good() });
const template16a = Comp({ render: state.dynamic || good.good });
const template17 = Comp({ render: state.dynamic ? Comp({}) : Comp({}) });
const template18 = Comp({ get children() {
	return state.dynamic ? Comp({}) : Comp({});
} });
const template19 = _$ssr([
	"<div",
	" innerHTML=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(state.dynamic ? <Comp /> : <Comp />, true));
const template20 = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$scope(() => {
	return _$escape(state.dynamic ? Comp({}) : Comp({}));
}));
const template21 = Comp({ render: state?.dynamic ? "a" : "b" });
const template22 = Comp({ get children() {
	return state?.dynamic ? "a" : "b";
} });
const template23 = _$ssr([
	"<div",
	" innerHTML=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(state?.dynamic ? "a" : "b", true));
const template24 = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$escape(state?.dynamic ? "a" : "b"));
const template25 = Comp({ render: state.dynamic ?? Comp({}) });
const template26 = Comp({ get children() {
	return state.dynamic ?? Comp({});
} });
const template27 = _$ssr([
	"<div",
	" innerHTML=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(state.dynamic ?? <Comp />, true));
const template28 = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$scope(() => {
	return _$escape(state.dynamic ?? Comp({}));
}));
const template29 = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$scope(() => {
	return _$escape((thing() && thing1()) ?? thing2() ?? thing3());
}));
const template29a = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$escape((thing.thing && thing1.thing1) ?? thing2.thing2 ?? thing3.thing3));
const template30 = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$scope(() => {
	return _$escape(thing() || thing1() || thing2());
}));
const template30a = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$escape(thing.thing || thing1.thing1 || thing2.thing2));
const template31 = Comp({ value: count() ? count() ? count() : count() : count() });
const template31a = Comp({ value: count.count ? count.count ? count.count : count.count : count.count });
const template32 = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$escape(something?.()));
const template33 = Comp({ get children() {
	return something?.();
} });
const template34 = simple ? good : bad;
const template35 = simple ? good() : bad;
const template35a = simple ? good.good : bad;
const template36 = state.dynamic ? good() : bad;
const template36a = state.dynamic ? good.good : bad;
const template37 = state.dynamic && good();
const template37a = state.dynamic && good.good;
const template38 = state.count > 5 ? state.dynamic ? best : good() : bad;
const template38a = state.count > 5 ? state.dynamic ? best : good.good : bad;
const template39 = state.dynamic && state.something && good();
const template39a = state.dynamic && state.something && good.good;
const template40 = state.dynamic && good() || bad;
const template40a = state.dynamic && good.good || bad;
const template41 = state.a ? "a" : state.b ? "b" : state.c ? "c" : "fallback";
const template42 = state.a ? a() : state.b ? b() : state.c ? "c" : "fallback";
const template42a = state.a ? a.a : state.b ? b.b : state.c ? "c" : "fallback";
const template43 = obj1.prop ? obj2.prop ? _$ssr(["<div", ">Output</div>"], _$ssrHydrationKey()) : null : null;
// statically boolean left: memo value IS the expression value, logical form kept
const template77 = state.count > 5 && good();
const template77a = !state.hidden && good.good;
