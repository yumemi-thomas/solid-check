import { createComponent as _$createComponent2 } from "r-custom";
import { template as _$template } from "r-dom";
import { insert as _$insert } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<span>Hello `);
var _tmpl$2 = /* @__PURE__ */ _$template(`<span> John`);
var _tmpl$3 = /* @__PURE__ */ _$template(`<span>Hello John`);
var _tmpl$4 = /* @__PURE__ */ _$template(`<span> `);
var _tmpl$5 = /* @__PURE__ */ _$template(`<span> <!> <!> `);
var _tmpl$6 = /* @__PURE__ */ _$template(`<span> <!> `);
var _tmpl$7 = /* @__PURE__ */ _$template(`<span>Hello`);
var _tmpl$8 = /* @__PURE__ */ _$template(`<span>&nbsp;&lt;Hi&gt;&nbsp;`);
var _tmpl$9 = /* @__PURE__ */ _$template(`<span>Hi&lt;script>alert();&lt;/script>`);
var _tmpl$10 = /* @__PURE__ */ _$template(`<span>4 + 5 = `);
var _tmpl$11 = /* @__PURE__ */ _$template(`<div>
d`);
var _tmpl$12 = /* @__PURE__ */ _$template(`<div>`);
var _tmpl$13 = /* @__PURE__ */ _$template(`<div normal=Search… title=Search&amp;hellip;>`);
const trailing = _tmpl$();
const leading = _tmpl$2();
/* prettier-ignore */
const extraSpaces = _tmpl$3();
const trailingExpr = (() => {
	var _el$4 = _tmpl$();
	_$insert(_el$4, name);
	return _el$4;
})();
const leadingExpr = (() => {
	var _el$5 = _tmpl$2();
	_$insert(_el$5, greeting, _el$5.firstChild);
	return _el$5;
})();
/* prettier-ignore */
const multiExpr = (() => {
	var _el$6 = _tmpl$4();
	_$insert(_el$6, greeting, _el$6.firstChild);
	_$insert(_el$6, name);
	return _el$6;
})();
/* prettier-ignore */
const multiExprSpaced = (() => {
	var _el$7 = _tmpl$5();
	var _el$8 = _el$7.firstChild.nextSibling;
	_$insert(_el$7, greeting, _el$8);
	var _el$9 = _el$7.firstChild.nextSibling.nextSibling.nextSibling;
	_$insert(_el$7, name, _el$9);
	return _el$7;
})();
/* prettier-ignore */
const multiExprTogether = (() => {
	var _el$10 = _tmpl$6();
	var _el$11 = _el$10.firstChild.nextSibling;
	_$insert(_el$10, greeting, _el$11);
	_$insert(_el$10, name, _el$11);
	return _el$10;
})();
/* prettier-ignore */
const multiLine = _tmpl$7();
/* prettier-ignore */
const multiLineTrailingSpace = _tmpl$3();
/* prettier-ignore */
const multiLineNoTrailingSpace = _tmpl$3();
/* prettier-ignore */
const escape = _tmpl$8();
/* prettier-ignore */
const escape2 = _$createComponent2(Comp, { children: "&nbsp;&lt;Hi&gt;&nbsp;" });
/* prettier-ignore */
const escape3 = "&nbsp;&lt;Hi&gt;&nbsp;";
const injection = _tmpl$9();
let value = "World";
const evaluated = (() => {
	var _el$17 = _tmpl$();
	_$insert(_el$17, value + "!");
	return _el$17;
})();
let number = 4 + 5;
const evaluatedNonString = (() => {
	var _el$18 = _tmpl$10();
	_$insert(_el$18, number);
	return _el$18;
})();
const newLineLiteral = (() => {
	var _el$19 = _tmpl$11();
	_$insert(_el$19, s, _el$19.firstChild);
	return _el$19;
})();
const trailingSpace = (() => {
	var _el$20 = _tmpl$12();
	_$insert(_el$20, expr);
	return _el$20;
})();
const trailingSpaceComp = _$createComponent2(Comp, { get children() {
	return expr;
} });
const trailingSpaceFrag = expr;
const leadingSpaceElement = (() => {
	var _el$21 = _tmpl$4();
	_$insert(_el$21, expr);
	return _el$21;
})();
const leadingSpaceComponent = _$createComponent2(Div, { get children() {
	return [" ", expr];
} });
const leadingSpaceFragment = [" ", expr];
const trailingSpaceElement = (() => {
	var _el$22 = _tmpl$4();
	_$insert(_el$22, expr, _el$22.firstChild);
	return _el$22;
})();
const trailingSpaceComponent = _$createComponent2(Div, { get children() {
	return [expr, " "];
} });
const trailingSpaceFragment = [expr, " "];
const escapeAttribute = _tmpl$13();
const escapeCompAttribute = _$createComponent2(Div, {
	normal: "Search…",
	title: "Search&hellip;"
});
