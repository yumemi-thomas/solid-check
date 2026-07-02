import { escape as _$escape } from "r-server";
import { ssr as _$ssr } from "r-server";
const template1 = _$ssr(["<div>", "</div>"], _$escape(simple));
const template2 = _$ssr(["<div>", "</div>"], _$escape(state.dynamic));
const template3 = _$ssr(["<div>", "</div>"], _$escape(simple ? good : bad));
const template4 = _$ssr(["<div>", "</div>"], _$escape(simple ? good() : bad));
const template4a = _$ssr(["<div>", "</div>"], _$escape(simple ? good.good : bad));
const template5 = _$ssr(["<div>", "</div>"], _$escape(state.dynamic ? good() : bad));
const template5a = _$ssr(["<div>", "</div>"], _$escape(state.dynamic ? good.good : bad));
const template6 = _$ssr(["<div>", "</div>"], _$escape(state.dynamic && good()));
const template6a = _$ssr(["<div>", "</div>"], _$escape(state.dynamic && good.good));
const template7 = _$ssr(["<div>", "</div>"], _$escape(state.count > 5 ? state.dynamic ? best : good() : bad));
const template7a = _$ssr(["<div>", "</div>"], _$escape(state.count > 5 ? state.dynamic ? best : good.good : bad));
const template8 = _$ssr(["<div>", "</div>"], _$escape(state.dynamic && state.something && good()));
const template8a = _$ssr(["<div>", "</div>"], _$escape(state.dynamic && state.something && good.good));
const template9 = _$ssr(["<div>", "</div>"], _$escape(state.dynamic && good() || bad));
const template9a = _$ssr(["<div>", "</div>"], _$escape(state.dynamic && good.good || bad));
const template10 = _$ssr(["<div>", "</div>"], _$escape(state.a ? "a" : state.b ? "b" : state.c ? "c" : "fallback"));
const template11 = _$ssr(["<div>", "</div>"], _$escape(state.a ? a() : state.b ? b() : state.c ? "c" : "fallback"));
const template11a = _$ssr(["<div>", "</div>"], _$escape(state.a ? a.a : state.b ? b.b : state.c ? "c" : "fallback"));
const template12 = Comp({ render: state.dynamic ? good() : bad });
const template12a = Comp({ render: state.dynamic ? good.good : bad });
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
const template19 = _$ssr(["<div innerHTML=\"", "\"></div>"], _$escape(state.dynamic ? <Comp /> : <Comp />, true));
const template20 = _$ssr(["<div>", "</div>"], _$escape(state.dynamic ? Comp({}) : Comp({})));
const template21 = Comp({ render: state?.dynamic ? "a" : "b" });
const template22 = Comp({ get children() {
	return state?.dynamic ? "a" : "b";
} });
const template23 = _$ssr(["<div innerHTML=\"", "\"></div>"], _$escape(state?.dynamic ? "a" : "b", true));
const template24 = _$ssr(["<div>", "</div>"], _$escape(state?.dynamic ? "a" : "b"));
const template25 = Comp({ render: state.dynamic ?? Comp({}) });
const template26 = Comp({ get children() {
	return state.dynamic ?? Comp({});
} });
const template27 = _$ssr(["<div innerHTML=\"", "\"></div>"], _$escape(state.dynamic ?? <Comp />, true));
const template28 = _$ssr(["<div>", "</div>"], _$escape(state.dynamic ?? Comp({})));
const template29 = _$ssr(["<div>", "</div>"], _$escape((thing() && thing1()) ?? thing2() ?? thing3()));
const template29a = _$ssr(["<div>", "</div>"], _$escape((thing.thing && thing1.thing1) ?? thing2.thing2 ?? thing3.thing3));
const template30 = _$ssr(["<div>", "</div>"], _$escape(thing() || thing1() || thing2()));
const template30a = _$ssr(["<div>", "</div>"], _$escape(thing.thing || thing1.thing1 || thing2.thing2));
const template31 = Comp({ value: count() ? count() ? count() : count() : count() });
const template31a = Comp({ value: count.count ? count.count ? count.count : count.count : count.count });
const template32 = _$ssr(["<div>", "</div>"], _$escape(something?.()));
const template32a = _$ssr(["<div>", "</div>"], _$escape(something?.something));
const template33 = Comp({ get children() {
	return something?.();
} });
const template33a = Comp({ get children() {
	return something?.something;
} });
const template34 = simple ? good : bad;
const template35 = simple ? good() : bad;
const template35a = simple ? good.good : bad;
const template36 = state.dynamic ? good() : bad;
const template36a = state.dynamic ? good.good : bad;
const template37 = state.dynamic && good();
const template37a = state.dynamic && good.good;
const template38 = state.count > 5 ? state.dynamic ? best : good() : bad;
const template38a = state.count > 5 ? state.dynamic ? best : good.good : bad.bad;
const template39 = state.dynamic && state.something && good();
const template40 = state.dynamic && good() || bad;
const template40a = state.dynamic && good.good || bad;
const template41 = state.a ? "a" : state.b ? "b" : state.c ? "c" : "fallback";
const template42 = state.a ? a() : state.b ? b() : state.c ? "c" : "fallback";
const template42a = state.a ? a.a : state.b ? b.b : state.c ? "c" : "fallback";
const template43 = obj1.prop ? obj2.prop ? _$ssr("<div>Output</div>") : null : null;
// single-significant-child fragment in element slot — outer _$escape wrap
// is skipped because the fragment compiles to a self-escaping form.
const template44 = _$ssr(["<div>", "</div>"], _$escape(cond && state.text));
const template45 = _$ssr(["<div>", "</div>"], _$escape(cond ? state.a : state.b));
const template46 = _$ssr(["<div>", "</div>"], _$escape(cond && _$ssr("<span>s</span>")));
const template47 = _$ssr(["<div>", "</div>"], _$escape(cond ? _$ssr("<span>a</span>") : _$ssr("<span>b</span>")));
// component inside fragment must keep the outer _$escape wrap because a
// component call can return any runtime type, including a raw string.
const template48 = _$ssr(["<div>", "</div>"], _$escape(cond && Comp({})));
// mixed fragment content keeps the outer wrap — predicate is conservative
// and only skips when exactly one significant child is provably safe.
const template49 = _$ssr(["<div>", "</div>"], _$escape(cond && ["hello ", state.text]));
