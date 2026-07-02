import { createTextNode as _$createTextNode } from "r-custom";
import { mergeProps as _$mergeProps } from "r-custom";
import { spread as _$spread } from "r-custom";
import { insertNode as _$insertNode } from "r-custom";
import { setProp as _$setProp } from "r-custom";
import { createElement as _$createElement } from "r-custom";
import { binding } from "somewhere";
function refFn() {}
const refConst = null;
const selected = true;
let link;
var _el$ = _$createElement("div");
_$spread(_el$, _$mergeProps({ id: "main" }, results, { style: { color } }), true);
var _el$2 = _$createElement("h1");
_$spread(_el$2, _$mergeProps({ class: "base" }, results(), {
	disabled: true,
	readonly: "",
	title: welcoming(),
	style: {
		"background-color": color(),
		"margin-right": "40px"
	},
	class: ["base", {
		dynamic: dynamic(),
		selected
	}]
}), true);
var _el$3 = _$createElement("a", {
	href: "/",
	readonly: value
});
_$setProp(_el$3, "ref", link);
_$insertNode(_el$3, _$createTextNode("Welcome"));
_$insertNode(_el$2, _el$3);
_$insertNode(_el$, _el$2);
const template = _el$;
var _el$4 = _$createElement("div");
_$spread(_el$4, getProps("test"), true);
var _el$5 = _$createElement("div", { textContent: rowId });
_$insertNode(_el$4, _el$5);
var _el$6 = _$createElement("div");
_$setProp(_el$6, "textContent", row.label);
_$insertNode(_el$4, _el$6);
var _el$7 = _$createElement("div", { innerHTML: "<div/>" });
_$insertNode(_el$4, _el$7);
const template2 = _el$4;
var _el$8 = _$createElement("div", {
	id: state.id,
	style: (	/*@static*/ { "background-color": state.color }),
	textContent: state.content
});
_$setProp(_el$8, "name", state.name);
const template3 = _el$8;
var _el$9 = _$createElement("div", { class: { "ccc:ddd": true } });
_$setProp(_el$9, "className", state.class);
const template4 = _el$9;
var _el$10 = _$createElement("div", {
	class: "a",
	className: "b"
});
const template5 = _el$10;
var _el$11 = _$createElement("div", { textContent: "Hi" });
_$setProp(_el$11, "style", someStyle());
const template6 = _el$11;
var _el$12 = _$createElement("div");
_$setProp(_el$12, "style", {
	"background-color": color(),
	"margin-right": "40px",
	...props.style
});
const template7 = _el$12;
let refTarget;
var _el$13 = _$createElement("div");
_$setProp(_el$13, "ref", refTarget);
const template8 = _el$13;
var _el$14 = _$createElement("div");
_$setProp(_el$14, "ref", (e) => console.log(e));
const template9 = _el$14;
var _el$15 = _$createElement("div");
_$setProp(_el$15, "ref", refFactory());
const template10 = _el$15;
var _el$16 = _$createElement("div", { "prop:htmlFor": thing });
const template12 = _el$16;
var _el$17 = _$createElement("input", {
	type: "checkbox",
	checked: true
});
const template13 = _el$17;
var _el$18 = _$createElement("input", { type: "checkbox" });
_$setProp(_el$18, "checked", state.visible);
const template14 = _el$18;
var _el$19 = _$createElement("div", { class: "`a" });
_$insertNode(_el$19, _$createTextNode("`$`"));
const template15 = _el$19;
var _el$20 = _$createElement("button", {
	class: ["static", { hi: "k" }],
	type: "button"
});
_$insertNode(_el$20, _$createTextNode("Write"));
const template16 = _el$20;
var _el$21 = _$createElement("button", {
	class: {
		a: true,
		b: true,
		c: true
	},
	onClick: increment
});
_$insertNode(_el$21, _$createTextNode("Hi"));
const template17 = _el$21;
var _el$22 = _$createElement("div");
_$spread(_el$22, { get [key()]() {
	return props.value;
} }, false);
const template18 = _el$22;
var _el$23 = _$createElement("div");
_$setProp(_el$23, "style", {
	a: "static",
	...rest
});
const template19 = _el$23;
var _el$24 = _$createElement("div");
_$setProp(_el$24, "ref", a().b.c);
const template21 = _el$24;
var _el$25 = _$createElement("div");
_$setProp(_el$25, "ref", a().b?.c);
const template22 = _el$25;
var _el$26 = _$createElement("div");
_$setProp(_el$26, "ref", a() ? b : c);
const template23 = _el$26;
var _el$27 = _$createElement("div");
_$setProp(_el$27, "ref", a() ?? b);
const template24 = _el$27;
var _el$28 = _$createElement("div");
_$setProp(_el$28, "ref", binding);
const template25 = _el$28;
var _el$29 = _$createElement("div");
_$setProp(_el$29, "ref", binding.prop);
const template26 = _el$29;
var _el$30 = _$createElement("div");
_$setProp(_el$30, "ref", refFn);
const template27 = _el$30;
var _el$31 = _$createElement("div");
_$setProp(_el$31, "ref", refConst);
const template28 = _el$31;
var _el$32 = _$createElement("div");
_$setProp(_el$32, "ref", refUnknown);
const template29 = _el$32;
