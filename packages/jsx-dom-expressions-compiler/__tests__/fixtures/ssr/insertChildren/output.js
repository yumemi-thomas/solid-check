import { escape as _$escape } from "r-server";
import { ssr as _$ssr } from "r-server";
import { ssrElement as _$ssrElement } from "r-server";
import { mergeProps as _$mergeProps } from "r-server";
const children = _$ssr("<div></div>");
const dynamic = { children };
const template = Module({ children });
const template2 = _$ssr(["<module children=\"", "\"></module>"], _$escape(children, true));
const template3 = _$ssr(["<module children=\"", "\">Hello</module>"], _$escape(children, true));
const template4 = _$ssr([
	"<module children=\"",
	"\">",
	"</module>"
], _$escape(children, true), Hello({}));
const template5 = _$ssr(["<module children=\"", "\"></module>"], _$escape(dynamic.children, true));
const template6 = Module({ get children() {
	return dynamic.children;
} });
const template7 = _$ssrElement("module", dynamic, undefined, false);
const template8 = _$ssrElement("module", dynamic, "Hello", false);
const template9 = _$ssrElement("module", dynamic, dynamic.children, false);
const template10 = Module(_$mergeProps(dynamic, { children: "Hello" }));
const template11 = _$ssr(["<module children=\"", "\"></module>"], _$escape(
	/*@static*/
	state.children,
	true
));
const template12 = Module({ children: state.children });
const template13 = _$ssr(["<module>", "</module>"], _$escape(children));
const template14 = Module({ get children() {
	return children;
} });
const template15 = _$ssr(["<module>", "</module>"], _$escape(dynamic.children));
const template16 = Module({ get children() {
	return dynamic.children;
} });
const template18 = _$ssr(["<module>Hi ", "</module>"], _$escape(children));
const template19 = Module({ get children() {
	return ["Hi ", children];
} });
const template20 = _$ssr(["<module>", "</module>"], _$escape(children()));
const template21 = Module({ get children() {
	return children();
} });
const template22 = _$ssr(["<module>", "</module>"], _$escape(state.children()));
const template23 = Module({ get children() {
	return state.children();
} });
const template24 = _$ssrElement("module", dynamic, ["Hi", dynamic.children], false);
const tiles = [];
tiles.push(_$ssr("<div>Test 1</div>"));
const template25 = _$ssr(["<div>", "</div>"], _$escape(tiles));
const comma = _$ssr(["<div>", "</div>"], _$escape((expression(), "static")));
const double = _$ssr(["<div>", "</div>"], _$escape(children()()));
