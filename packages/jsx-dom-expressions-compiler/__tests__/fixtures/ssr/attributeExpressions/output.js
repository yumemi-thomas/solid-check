import { escape as _$escape } from "r-server";
import { ssr as _$ssr } from "r-server";
import { ssrElement as _$ssrElement } from "r-server";
import { mergeProps as _$mergeProps } from "r-server";
import * as styles from "./styles.module.css";
import { binding } from "somewhere";
function refFn() {}
const refConst = null;
const selected = true;
let id = "my-h1";
let link;
const template = _$ssrElement("div", _$mergeProps({ id: "main" }, results, {
	class: { selected: unknown },
	style: { color }
}), _$ssrElement("h1", _$mergeProps({ id }, results(), {
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
}), _$ssr([
	"<a href=\"/\" ref=\"",
	"\" class=\"",
	"\">Welcome</a>"
], _$escape(link, true), _$escape({ "ccc ddd": true }, true)), false), false);
const template2 = _$ssrElement("div", getProps("test"), [
	_$ssr(["<div textContent=\"", "\"></div>"], _$escape(rowId, true)),
	_$ssr(["<div textContent=\"", "\"></div>"], _$escape(row.label, true)),
	_$ssr("<div innerHTML=\"&lt;div/>\"></div>")
], false);
const template3 = _$ssr([
	"<div foo id=\"",
	"\" style=\"",
	"\" name=\"",
	"\" textContent=\"",
	"\"></div>"
], _$escape(
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
	"<div className=\"",
	"\" class=\"",
	"\"></div>"
], _$escape(state.class, true), _$escape({ "ccc:ddd": true }, true));
const template5 = _$ssr("<div class=\"a\" className=\"b\"></div>");
const template6 = _$ssr(["<div style=\"", "\" textContent=\"Hi\"></div>"], _$escape(someStyle(), true));
let undefVar;
const template7 = _$ssr([
	"<div style=\"",
	"\" class=\"",
	"\"></div>"
], _$escape({
	"background-color": color(),
	"margin-right": "40px",
	...props.style
}, true), _$escape({ "other-class2": undefVar }, true));
let refTarget;
const template8 = _$ssr(["<div ref=\"", "\"></div>"], _$escape(refTarget, true));
const template9 = _$ssr(["<div ref=\"", "\"></div>"], _$escape((e) => console.log(e), true));
const template10 = _$ssr(["<div ref=\"", "\"></div>"], _$escape(refFactory(), true));
const template12 = _$ssr("<div onclick=\"console.log('hi')\"></div>");
const template13 = _$ssr("<input type=\"checkbox\" checked=\"true\">");
const template14 = _$ssr(["<input type=\"checkbox\" checked=\"", "\">"], _$escape(state.visible, true));
const template15 = _$ssr("<div class=\"`a\">`$`</div>");
const template16 = _$ssr(["<button class=\"", "\" type=\"button\">Write</button>"], _$escape(["static", { hi: "k" }], true));
const template17 = _$ssr([
	"<button class=\"",
	"\" onClick=\"",
	"\">Hi</button>"
], _$escape({
	a: true,
	b: true,
	c: true
}, true), _$escape(increment, true));
const template18 = _$ssrElement("div", { get [key()]() {
	return props.value;
} }, undefined, false);
const template19 = _$ssr(["<div class=\"", "\"></div>"], _$escape([{ "bg-red-500": true }, "flex flex-col"], true));
const template20 = _$ssr([
	"<div><input value=\"",
	"\" min=\"",
	"\" max=\"",
	"\" onInput=\"",
	"\" readonly=\"\"><input checked=\"",
	"\" min=\"",
	"\" max=\"",
	"\" onInput=\"",
	"\" readonly=\"",
	"\"></div>"
], _$escape(s(), true), _$escape(min(), true), _$escape(max(), true), _$escape(doSomething, true), _$escape(s2(), true), _$escape(min(), true), _$escape(max(), true), _$escape(doSomethingElse, true), _$escape(value, true));
const template21 = _$ssr(["<div style=\"", "\"></div>"], _$escape({
	a: "static",
	...rest
}, true));
const template22 = _$ssr("<div data=\"&quot;hi&quot;\" data2=\"&quot;\"></div>");
const template23 = _$ssr([
	"<div disabled=\"",
	"\">",
	"</div>"
], _$escape("t" in test, true), _$escape("t" in test && "true"));
const template24 = _$ssrElement("a", _$mergeProps(props, { something: true }), undefined, false);
const template25 = _$ssr([
	"<div>",
	"",
	"</div>"
], _$escape(props.children), _$ssrElement("a", _$mergeProps(props, { something: true }), undefined, false));
const template26 = _$ssrElement("div", _$mergeProps({
	start: "Hi",
	middle
}, spread), "Hi", false);
const template27 = _$ssrElement("div", _$mergeProps({ start: "Hi" }, first, { middle }, second), "Hi", false);
const template28 = _$ssrElement("label", api(), [
	_$ssrElement("span", api(), ["Input is ", api() ? "checked" : "unchecked"], false),
	_$ssrElement("input", api(), undefined, false),
	_$ssrElement("div", api(), undefined, false)
], false);
const template29 = _$ssr([
	"<div attribute=\"",
	"\">",
	"</div>"
], _$escape(!!someValue, true), _$escape(!!someValue));
const template30 = _$ssr("<div class=\"class1 class2\n    class3 class4\n    class5 class6\" style=\"color: red;\n    background-color: blue !important;\n    border: 1px solid black;\n    font-size: 12px;\" random=\"random1 random2\n    random3 random4\"></div>");
const template31 = _$ssr(["<div style=\"", "\"></div>"], _$escape({ "background-color": getStore.itemProperties.color }, true));
const template32 = _$ssr(["<div style=\"", "\"></div>"], _$escape({ "background-color": undefined }, true));
const template33 = [
	_$ssr(["<button class=\"", "\"></button>"], _$escape(styles.button, true)),
	_$ssr(["<button class=\"", "\"></button>"], _$escape(styles["foo--bar"], true)),
	_$ssr(["<button class=\"", "\"></button>"], _$escape(styles.foo.bar, true)),
	_$ssr(["<button class=\"", "\"></button>"], _$escape(styles[foo()], true))
];
const template35 = _$ssr(["<div ref=\"", "\"></div>"], _$escape(a().b.c, true));
const template36 = _$ssr(["<div ref=\"", "\"></div>"], _$escape(a().b?.c, true));
const template37 = _$ssr(["<div ref=\"", "\"></div>"], _$escape(a() ? b : c, true));
const template38 = _$ssr(["<div ref=\"", "\"></div>"], _$escape(a() ?? b, true));
const template39 = _$ssr("<input value=\"10\">");
const template40 = _$ssr(["<div style=\"", "\"></div>"], _$escape({ color: a() }, true));
const template41 = _$ssr([
	"<select value=\"",
	"\"><option value=\"",
	"\">Red</option><option value=\"",
	"\">Blue</option></select>"
], _$escape(state.color, true), _$escape(Color.Red, true), _$escape(Color.Blue, true));
const template42 = _$ssr("<img src=\"\">");
const template43 = _$ssr("<div><img src=\"\"></div>");
const template44 = _$ssr("<img src=\"\" loading=\"lazy\">");
const template45 = _$ssr("<div><img src=\"\" loading=\"lazy\"></div>");
const template46 = _$ssr("<iframe src=\"\"></iframe>");
const template47 = _$ssr("<div><iframe src=\"\"></iframe></div>");
const template48 = _$ssr("<iframe src=\"\" loading=\"lazy\"></iframe>");
const template49 = _$ssr("<div><iframe src=\"\" loading=\"lazy\"></iframe></div>");
const template50 = _$ssr("<div title=\"&lt;u>data&lt;/u>\"></div>");
const template51 = _$ssr(["<div ref=\"", "\"></div>"], _$escape(binding, true));
const template52 = _$ssr(["<div ref=\"", "\"></div>"], _$escape(binding.prop, true));
const template53 = _$ssr(["<div ref=\"", "\"></div>"], _$escape(refFn, true));
const template54 = _$ssr(["<div ref=\"", "\"></div>"], _$escape(refConst, true));
const template55 = _$ssr(["<div ref=\"", "\"></div>"], _$escape(refUnknown, true));
const template56 = _$ssr("<div true=\"true\" truestr=\"true\" truestrjs=\"true\"></div>");
const template57 = _$ssr("<div false=\"false\" falsestr=\"false\" falsestrjs=\"false\"></div>");
const template58 = _$ssr("<div></div>");
const template59 = _$ssr("<div true=\"true\" false=\"false\"></div>");
const template60 = _$ssr([
	"<div a b=\"\" c=\"\" d=\"true\" e=\"false\" f=\"0\" g=\"\" h=\"\" i=\"",
	"\" j=\"null\" k=\"",
	"\" l></div>"
], _$escape(undefined, true), _$escape(void 0, true));
const template61 = _$ssr("<math display=\"block\"><mrow></mrow></math>");
const template62 = _$ssr("<mrow><mi>x</mi><mo>=</mo></mrow>");
const template63 = _$ssr(["<div style=\"", "\"></div>"], _$escape({ "background": "red" }, true));
const template64 = _$ssr(["<div style=\"", "\"></div>"], _$escape({
	"background": "red",
	"color": "green",
	"margin": 3,
	"padding": .4
}, true));
const template65 = _$ssr(["<div style=\"", "\"></div>"], _$escape({
	"background": "red",
	"color": "green",
	"border": undefined
}, true));
const template66 = _$ssr(["<div style=\"", "\"></div>"], _$escape({
	"background": "red",
	"color": "green",
	"border": signal()
}, true));
const template67 = _$ssr(["<div style=\"", "\"></div>"], _$escape({
	"background": "red",
	"color": "green",
	"border": somevalue
}, true));
const template68 = _$ssr(["<div style=\"", "\"></div>"], _$escape({
	"background": "red",
	"color": "green",
	"border": some.access
}, true));
const template69 = _$ssr(["<div style=\"", "\"></div>"], _$escape({
	"background": "red",
	"color": "green",
	"border": null
}, true));
const template70 = _$ssr(["<video playsinline=\"", "\"></video>"], _$escape(value, true));
const template71 = _$ssr("<video playsinline=\"true\"></video>");
const template72 = _$ssr("<video playsinline=\"false\"></video>");
const template73 = _$ssr("<video poster=\"1.jpg\"></video>");
const template74 = _$ssr("<div><video poster=\"1.jpg\"></video></div>");
const template75 = _$ssr("<video></video>");
const template76 = _$ssr("<div><video></video></div>");
// STATIC TESTS
const template77 = _$ssr(["<div style=\"", "\"></div>"], _$escape(
	/*@static*/
	{
		width: props.width,
		height: props.height
	},
	true
));
const template78 = _$ssr([
	"<div style=\"",
	"\" something=\"",
	"\"></div>"
], _$escape(
	/*@static*/
	{
		width: props.width,
		height: props.height
	},
	true
), _$escape(color(), true));
const template79 = _$ssr([
	"<div style=\"",
	"\" something=\"",
	"\"></div>"
], _$escape({
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
const template82 = _$ssrElement("div", _$mergeProps(propsSpread, {
	"data-dynamic": color(),
	"data-static": color()
}), undefined, false);
const template83 = _$ssrElement("div", _$mergeProps(propsSpread, {
	"data-dynamic": color(),
	"data-static": color()
}), undefined, false);
const template84 = _$ssrElement("div", _$mergeProps(propsSpread1, propsSpread2, propsSpread3, {
	"data-dynamic": color(),
	"data-static": color()
}), undefined, false);
// STATIC PROPERTY OF OBJECT ACCESS
// https://github.com/ryansolid/dom-expressions/issues/252#issuecomment-1572220563
const styleProp = { style: {
	width: props.width,
	height: props.height
} };
const template85 = _$ssr(["<div style=\"", "\"></div>"], _$escape(
	/* @static */
	styleProp.style,
	true
));
const template86 = _$ssr(["<div style=\"", "\"></div>"], _$escape(styleProp.style, true));
const style = {
	background: "red",
	border: "solid black " + count() + "px"
};
const template87 = _$ssr([
	"<button type=\"button\" aria-label=\"",
	"\" style=\"",
	"\" class=\"",
	"\">",
	"</button>"
], _$escape(count(), true), _$escape(style, true), _$escape(style, true), _$escape(count()));
const template88 = _$ssr([
	"<button type=\"button\" aria-label=\"",
	"\" style=\"",
	"\" class=\"",
	"\">",
	"</button>"
], _$escape(count(), true), _$escape(
	/* @static*/
	style,
	true
), _$escape(
	/* @static*/
	style,
	true
), _$escape(count()));
const css = () => "&{color:red}";
const template89 = [
	_$ssr(["<style>", "</style>"], _$escape(css())),
	_$ssr(["<style children=\"", "\"></style>"], _$escape(css(), true)),
	_$ssr(["<style innerHTML=\"", "\"></style>"], _$escape(css(), true)),
	_$ssr(["<style innerText=\"", "\"></style>"], _$escape(css(), true)),
	_$ssr(["<style textContent=\"", "\"></style>"], _$escape(css(), true))
];
const styleProps = { children: css };
const template90 = [
	_$ssrElement("style", styleProps(), css(), false),
	_$ssrElement("style", _$mergeProps(styleProps(), { children: css() }), undefined, false),
	_$ssrElement("style", _$mergeProps(styleProps(), { innerHTML: css() }), undefined, false),
	_$ssrElement("style", _$mergeProps(styleProps(), { innerText: css() }), undefined, false),
	_$ssrElement("style", _$mergeProps(styleProps(), { textContent: css() }), undefined, false)
];
const nope = () => undefined;
const template91 = _$ssr(["<div class=\"bg-(--bg)\" style=\"", "\"></div>"], _$escape({ "--bg": nope() }, true));
const template92 = _$ssr(["<div style=\"", "\"></div>"], _$escape({}, true));
const template93 = _$ssr(["<div data-test=\"", "\"></div>"], _$escape(state.flag || undefined, true));
function Progress(props) {
	return _$ssr(["<div class=\"progress-fill\" style=\"", "\"></div>"], _$escape({ [props.orientation === "y" ? "height" : "width"]: `${props.value * 100}%` }, true));
}
const template94 = _$ssr([
	"<div><textarea value=\"",
	"\"></textarea><textarea value=\"",
	"\">",
	"</textarea><textarea value=\"",
	"\"></textarea><textarea></textarea><textarea>",
	"</textarea><textarea value=\"static content\"></textarea><textarea value=\"static content\">I get replaced</textarea></div>"
], _$escape(dynamicProperty(), true), _$escape(dynamicProperty(), true), _$escape(dynamicContent()), _$escape(dynamicContent(), true), _$escape(dynamicContent()));
const template95 = _$ssr([
	"<div><video muted=\"true\"></video><video muted=\"false\"></video><video defaultMuted=\"false\" muted=\"",
	"\"></video><video defaultMuted=\"true\" muted=\"",
	"\"></video><video defaultMuted=\"",
	"\" muted=\"",
	"\"></video><video src=\"test.mp4\" muted></video></div>"
], _$escape(dynamicProperty(), true), _$escape(dynamicProperty(), true), _$escape(dynamicAttribute(), true), _$escape(dynamicProperty(), true));
