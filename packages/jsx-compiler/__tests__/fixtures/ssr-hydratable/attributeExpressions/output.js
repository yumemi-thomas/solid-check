import { scope as _$scope } from "r-server";
import { escape as _$escape } from "r-server";
import { ssr as _$ssr } from "r-server";
import { ssrHydrationKey as _$ssrHydrationKey } from "r-server";
import { ssrElement as _$ssrElement } from "r-server";
import { mergeProps as _$mergeProps } from "r-server";
import * as styles from "./styles.module.css";
import { binding } from "somewhere";
function refFn() {}
const refConst = null;
const selected = true;
let id = "my-h1";
let link;
const template = _$ssrElement("div", () => {
	return _$mergeProps({ id: "main" }, results, {
		class: { selected: unknown },
		style: { color }
	});
}, _$ssrElement("h1", () => {
	return _$mergeProps({ id }, results(), {
		foo: true,
		disabled: true,
		title: welcoming(),
		style: {
			"background-color": color(),
			"margin-right": "40px"
		},
		class: ["base", {
			dynamic: dynamic(),
			selected
		}]
	});
}, _$ssr([
	"<a",
	" href=\"/\" ref=\"",
	"\" class=\"",
	"\">Welcome</a>"
], _$ssrHydrationKey(), _$escape(link, true), _$escape({ "ccc ddd": true }, true)), false), false);
const template2 = _$ssrElement("div", getProps("test"), [
	_$ssr([
		"<div",
		" textContent=\"",
		"\"></div>"
	], _$ssrHydrationKey(), _$escape(rowId, true)),
	_$ssr([
		"<div",
		" textContent=\"",
		"\"></div>"
	], _$ssrHydrationKey(), _$escape(row.label, true)),
	_$ssr(["<div", " innerHTML=\"<div/>\"></div>"], _$ssrHydrationKey())
], false);
const template3 = _$ssr([
	"<div",
	" foo id=\"",
	"\" style=\"",
	"\" name=\"",
	"\" textContent=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(
	/*@static*/
	state.id,
	true
), _$escape(
	/*@static*/
	{ "background-color": state.color },
	true
), _$escape(state.name, true), _$escape(
	/*@static*/
	state.content,
	true
));
const template4 = _$ssr([
	"<div",
	" className=\"",
	"\" class=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(state.class, true), _$escape({ "ccc:ddd": true }, true));
const template5 = _$ssr(["<div", " class=\"a\" className=\"b\"></div>"], _$ssrHydrationKey());
const template6 = _$ssr([
	"<div",
	" style=\"",
	"\" textContent=\"Hi\"></div>"
], _$ssrHydrationKey(), _$escape(someStyle(), true));
let undefVar;
const template7 = _$ssr([
	"<div",
	" style=\"",
	"\" class=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape({
	"background-color": color(),
	"margin-right": "40px",
	...props.style
}, true), _$escape({ "other-class2": undefVar }, true));
let refTarget;
const template8 = _$ssr([
	"<div",
	" ref=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(refTarget, true));
const template9 = _$ssr([
	"<div",
	" ref=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape((e) => console.log(e), true));
const template10 = _$ssr([
	"<div",
	" ref=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(refFactory(), true));
const template12 = _$ssr(["<div", " onclick=\"console.log('hi')\"></div>"], _$ssrHydrationKey());
const template13 = _$ssr(["<input", " type=\"checkbox\" checked>"], _$ssrHydrationKey());
const template14 = _$ssr([
	"<input",
	" type=\"checkbox\" checked=\"",
	"\">"
], _$ssrHydrationKey(), _$escape(state.visible, true));
const template15 = _$ssr(["<div", " class=\"`a\">`$`</div>"], _$ssrHydrationKey());
const template16 = _$ssr([
	"<button",
	" class=\"",
	"\" type=\"button\">Write</button>"
], _$ssrHydrationKey(), _$escape(["static", { hi: "k" }], true));
const template17 = _$ssr([
	"<button",
	" class=\"",
	"\" onClick=\"",
	"\">Hi</button>"
], _$ssrHydrationKey(), _$escape({
	a: true,
	b: true,
	c: true
}, true), _$escape(increment, true));
const template18 = _$ssrElement("div", { get [key()]() {
	return props.value;
} }, undefined, false);
const template19 = _$ssr([
	"<div",
	" class=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape([{ "bg-red-500": true }, "flex flex-col"], true));
const template20 = _$ssr([
	"<div",
	"><input value=\"",
	"\" min=\"",
	"\" max=\"",
	"\" onInput=\"",
	"\" readonly><input checked=\"",
	"\" min=\"",
	"\" max=\"",
	"\" onInput=\"",
	"\" readonly=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(s(), true), _$escape(min(), true), _$escape(max(), true), _$escape(doSomething, true), _$escape(s2(), true), _$escape(min(), true), _$escape(max(), true), _$escape(doSomethingElse, true), _$escape(value, true));
const template21 = _$ssr([
	"<div",
	" style=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape({
	a: "static",
	...rest
}, true));
const template22 = _$ssr(["<div", " data=\"&quot;hi&quot;\" data2=\"&quot;\"></div>"], _$ssrHydrationKey());
const template23 = _$ssr([
	"<div",
	" disabled=\"",
	"\">",
	"</div>"
], _$ssrHydrationKey(), _$escape("t" in test, true), _$escape("t" in test && "true"));
const template24 = _$ssrElement("a", () => {
	return _$mergeProps(props, { something: true });
}, undefined, false);
const template25 = _$ssr([
	"<div",
	">",
	"",
	"</div>"
], _$ssrHydrationKey(), _$scope(() => {
	return _$escape(props.children);
}), _$ssrElement("a", () => {
	return _$mergeProps(props, { something: true });
}, undefined, false));
const template26 = _$ssrElement("div", () => {
	return _$mergeProps({
		start: "Hi",
		middle
	}, spread);
}, "Hi", false);
const template27 = _$ssrElement("div", () => {
	return _$mergeProps({ start: "Hi" }, first, { middle }, second);
}, "Hi", false);
const template28 = _$ssrElement("label", api(), [
	_$ssrElement("span", api(), ["Input is ", api() ? "checked" : "unchecked"], false),
	_$ssrElement("input", api(), undefined, false),
	_$ssrElement("div", api(), undefined, false)
], false);
const template29 = _$ssr([
	"<div",
	" attribute=\"",
	"\">",
	"</div>"
], _$ssrHydrationKey(), _$escape(!!someValue, true), _$escape(!!someValue));
const template30 = _$ssr(["<div", " class=\"class1 class2\n    class3 class4\n    class5 class6\" style=\"color: red;\n    background-color: blue !important;\n    border: 1px solid black;\n    font-size: 12px;\" random=\"random1 random2\n    random3 random4\"></div>"], _$ssrHydrationKey());
const template31 = _$ssr([
	"<div",
	" style=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape({ "background-color": getStore.itemProperties.color }, true));
const template32 = _$ssr([
	"<div",
	" style=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape({ "background-color": undefined }, true));
const template33 = [
	_$ssr([
		"<button",
		" class=\"",
		"\"></button>"
	], _$ssrHydrationKey(), _$escape(styles.button, true)),
	_$ssr([
		"<button",
		" class=\"",
		"\"></button>"
	], _$ssrHydrationKey(), _$escape(styles["foo--bar"], true)),
	_$ssr([
		"<button",
		" class=\"",
		"\"></button>"
	], _$ssrHydrationKey(), _$escape(styles.foo.bar, true)),
	_$ssr([
		"<button",
		" class=\"",
		"\"></button>"
	], _$ssrHydrationKey(), _$escape(styles[foo()], true))
];
const template35 = _$ssr([
	"<div",
	" ref=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(a().b.c, true));
const template36 = _$ssr([
	"<div",
	" ref=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(a().b?.c, true));
const template37 = _$ssr([
	"<div",
	" ref=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(a() ? b : c, true));
const template38 = _$ssr([
	"<div",
	" ref=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(a() ?? b, true));
const template39 = _$ssr(["<input", " value=\"10\">"], _$ssrHydrationKey());
const template40 = _$ssr([
	"<div",
	" style=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape({ color: a() }, true));
const template41 = _$ssr([
	"<select",
	" value=\"",
	"\"><option value=\"",
	"\">Red</option><option value=\"",
	"\">Blue</option></select>"
], _$ssrHydrationKey(), _$escape(state.color, true), _$escape(Color.Red, true), _$escape(Color.Blue, true));
const template42 = _$ssr(["<img", " src>"], _$ssrHydrationKey());
const template43 = _$ssr(["<div", "><img src></div>"], _$ssrHydrationKey());
const template44 = _$ssr(["<img", " src loading=\"lazy\">"], _$ssrHydrationKey());
const template45 = _$ssr(["<div", "><img src loading=\"lazy\"></div>"], _$ssrHydrationKey());
const template46 = _$ssr(["<iframe", " src></iframe>"], _$ssrHydrationKey());
const template47 = _$ssr(["<div", "><iframe src></iframe></div>"], _$ssrHydrationKey());
const template48 = _$ssr(["<iframe", " src loading=\"lazy\"></iframe>"], _$ssrHydrationKey());
const template49 = _$ssr(["<div", "><iframe src loading=\"lazy\"></iframe></div>"], _$ssrHydrationKey());
const template50 = _$ssr(["<div", " title=\"<u>data</u>\"></div>"], _$ssrHydrationKey());
const template51 = _$ssr([
	"<div",
	" ref=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(binding, true));
const template52 = _$ssr([
	"<div",
	" ref=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(binding.prop, true));
const template53 = _$ssr([
	"<div",
	" ref=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(refFn, true));
const template54 = _$ssr([
	"<div",
	" ref=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(refConst, true));
const template55 = _$ssr([
	"<div",
	" ref=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(refUnknown, true));
const template56 = _$ssr(["<div", " true truestr=\"true\" truestrjs=\"true\"></div>"], _$ssrHydrationKey());
const template57 = _$ssr(["<div", " falsestr=\"false\" falsestrjs=\"false\"></div>"], _$ssrHydrationKey());
const template58 = _$ssr(["<div", "></div>"], _$ssrHydrationKey());
const template59 = _$ssr(["<div", " true></div>"], _$ssrHydrationKey());
const template60 = _$ssr([
	"<div",
	" a b c d f=\"0\" g h i=\"",
	"\" j=\"",
	"\" k=\"",
	"\" l></div>"
], _$ssrHydrationKey(), _$escape(undefined, true), _$escape(null, true), _$escape(void 0, true));
const template61 = _$ssr(["<math", " display=\"block\"><mrow></mrow></math>"], _$ssrHydrationKey());
const template62 = _$ssr(["<mrow", "><mi>x</mi><mo>=</mo></mrow>"], _$ssrHydrationKey());
const template63 = _$ssr([
	"<div",
	" style=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape({ "background": "red" }, true));
const template64 = _$ssr([
	"<div",
	" style=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape({
	"background": "red",
	"color": "green",
	"margin": 3,
	"padding": .4
}, true));
const template65 = _$ssr([
	"<div",
	" style=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape({
	"background": "red",
	"color": "green",
	"border": undefined
}, true));
const template66 = _$ssr([
	"<div",
	" style=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape({
	"background": "red",
	"color": "green",
	"border": signal()
}, true));
const template67 = _$ssr([
	"<div",
	" style=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape({
	"background": "red",
	"color": "green",
	"border": somevalue
}, true));
const template68 = _$ssr([
	"<div",
	" style=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape({
	"background": "red",
	"color": "green",
	"border": some.access
}, true));
const template69 = _$ssr([
	"<div",
	" style=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape({
	"background": "red",
	"color": "green",
	"border": null
}, true));
const template70 = _$ssr([
	"<video",
	" playsinline=\"",
	"\"></video>"
], _$ssrHydrationKey(), _$escape(value, true));
const template71 = _$ssr(["<video", " playsinline></video>"], _$ssrHydrationKey());
const template72 = _$ssr(["<video", "></video>"], _$ssrHydrationKey());
const template73 = _$ssr(["<video", " poster=\"1.jpg\"></video>"], _$ssrHydrationKey());
const template74 = _$ssr(["<div", "><video poster=\"1.jpg\"></video></div>"], _$ssrHydrationKey());
const template75 = _$ssr(["<video", "></video>"], _$ssrHydrationKey());
const template76 = _$ssr(["<div", "><video></video></div>"], _$ssrHydrationKey());
// STATIC TESTS
const template77 = _$ssr([
	"<div",
	" style=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(
	/*@static*/
	{
		width: props.width,
		height: props.height
	},
	true
));
const template78 = _$ssr([
	"<div",
	" style=\"",
	"\" something=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(
	/*@static*/
	{
		width: props.width,
		height: props.height
	},
	true
), _$escape(color(), true));
const template79 = _$ssr([
	"<div",
	" style=\"",
	"\" something=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape({
	width: props.width,
	height: props.height
}, true), _$escape(
	/*@static*/
	color(),
	true
));
// STATIC TESTS SPREADS
const propsSpread = {
	something: color(),
	style: {
		"background-color": color(),
		color: color(),
		"margin-right": props.right
	}
};
const template80 = _$ssrElement("div", propsSpread, undefined, false);
const template81 = _$ssrElement("div", propsSpread, undefined, false);
const template82 = _$ssrElement("div", () => {
	return _$mergeProps(propsSpread, {
		"data-dynamic": color(),
		"data-static": color()
	});
}, undefined, false);
const template83 = _$ssrElement("div", () => {
	return _$mergeProps(propsSpread, {
		"data-dynamic": color(),
		"data-static": color()
	});
}, undefined, false);
const template84 = _$ssrElement("div", () => {
	return _$mergeProps(propsSpread1, propsSpread2, propsSpread3, {
		"data-dynamic": color(),
		"data-static": color()
	});
}, undefined, false);
// STATIC PROPERTY OF OBJECT ACCESS
// https://github.com/ryansolid/dom-expressions/issues/252#issuecomment-1572220563
const styleProp = { style: {
	width: props.width,
	height: props.height
} };
const template85 = _$ssr([
	"<div",
	" style=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(
	/* @static */
	styleProp.style,
	true
));
const template86 = _$ssr([
	"<div",
	" style=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(styleProp.style, true));
const style = {
	background: "red",
	border: "solid black " + count() + "px"
};
const template87 = _$ssr([
	"<button",
	" type=\"button\" aria-label=\"",
	"\" style=\"",
	"\" class=\"",
	"\">",
	"</button>"
], _$ssrHydrationKey(), _$escape(count(), true), _$escape(style, true), _$escape(style, true), _$scope(() => {
	return _$escape(count());
}));
const template88 = _$ssr([
	"<button",
	" type=\"button\" aria-label=\"",
	"\" style=\"",
	"\" class=\"",
	"\">",
	"</button>"
], _$ssrHydrationKey(), _$escape(count(), true), _$escape(
	/* @static*/
	style,
	true
), _$escape(
	/* @static*/
	style,
	true
), _$scope(() => {
	return _$escape(count());
}));
const template89 = _$ssr([
	"<div",
	" style=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape({}, true));
const template90 = _$ssr([
	"<div",
	" data-test=\"",
	"\"></div>"
], _$ssrHydrationKey(), _$escape(state.flag || undefined, true));
const template91 = _$ssr([
	"<div",
	"><video muted></video><video></video><video muted=\"",
	"\"></video><video defaultMuted muted=\"",
	"\"></video><video defaultMuted=\"",
	"\" muted=\"",
	"\"></video><video src=\"test.mp4\" muted></video></div>"
], _$ssrHydrationKey(), _$escape(dynamicProperty(), true), _$escape(dynamicProperty(), true), _$escape(dynamicAttribute(), true), _$escape(dynamicProperty(), true));
function MyVideo() {
	return _$ssr(["<video", " src=\"test.mp4\" muted></video>"], _$ssrHydrationKey());
}
