import { template as _$template } from "r-dom";
import { getNextElement as _$getNextElement } from "r-dom";
import { insert as _$insert } from "r-dom";
import { createComponent as _$createComponent } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<span>Hello `);
var _tmpl$2 = /* @__PURE__ */ _$template(`<span> John`);
var _tmpl$3 = /* @__PURE__ */ _$template(`<span>Hello John`);
var _tmpl$4 = /* @__PURE__ */ _$template(`<span> `);
var _tmpl$5 = /* @__PURE__ */ _$template(`<span> <!> <!> `);
var _tmpl$6 = /* @__PURE__ */ _$template(`<span> <!> `);
var _tmpl$7 = /* @__PURE__ */ _$template(`<span>Hello`);
var _tmpl$8 = /* @__PURE__ */ _$template(`<span>&nbsp;&lt;Hi&gt;&nbsp;`);
var _tmpl$9 = /* @__PURE__ */ _$template(`<span>Hi&lt;script>alert();&lt;/script>`);
var _tmpl$10 = /* @__PURE__ */ _$template(`<span>Hello World!`);
var _tmpl$11 = /* @__PURE__ */ _$template(`<span>4 + 5 = 9`);
var _tmpl$12 = /* @__PURE__ */ _$template(`<div>
d`);
var _tmpl$13 = /* @__PURE__ */ _$template(`<div>`);
var _tmpl$14 = /* @__PURE__ */ _$template(`<div normal=Search… title=Search&amp;hellip;>`);
var _tmpl$15 = /* @__PURE__ */ _$template(`<div><div>`);
const trailing = _$getNextElement(_tmpl$);
const leading = _$getNextElement(_tmpl$2);
/* prettier-ignore */
const extraSpaces = _$getNextElement(_tmpl$3);
var _el$4 = _$getNextElement(_tmpl$);
_$insert(_el$4, name);
const trailingExpr = _el$4;
var _el$5 = _$getNextElement(_tmpl$2);
_$insert(_el$5, greeting, _el$5.firstChild);
const leadingExpr = _el$5;
var _el$6 = _$getNextElement(_tmpl$4);
_$insert(_el$6, greeting, _el$6.firstChild);
_$insert(_el$6, name);
/* prettier-ignore */
const multiExpr = _el$6;
var _el$7 = _$getNextElement(_tmpl$5);
var _el$8 = _el$7.firstChild.nextSibling;
_$insert(_el$7, greeting, _el$8);
var _el$9 = _el$7.firstChild.nextSibling.nextSibling.nextSibling;
_$insert(_el$7, name, _el$9);
/* prettier-ignore */
const multiExprSpaced = _el$7;
var _el$10 = _$getNextElement(_tmpl$6);
var _el$11 = _el$10.firstChild.nextSibling;
_$insert(_el$10, greeting, _el$11);
_$insert(_el$10, name, _el$11);
/* prettier-ignore */
const multiExprTogether = _el$10;
/* prettier-ignore */
const multiLine = _$getNextElement(_tmpl$7);
/* prettier-ignore */
const multiLineTrailingSpace = _$getNextElement(_tmpl$3);
/* prettier-ignore */
const multiLineNoTrailingSpace = _$getNextElement(_tmpl$3);
/* prettier-ignore */
const escape = _$getNextElement(_tmpl$8);
/* prettier-ignore */
const escape2 = _$createComponent(Comp, { children: "\xA0<Hi>\xA0" });
/* prettier-ignore */
const escape3 = "\xA0<Hi>\xA0";
const injection = _$getNextElement(_tmpl$9);
let value = "World";
const evaluated = _$getNextElement(_tmpl$10);
let number = 4 + 5;
const evaluatedNonString = _$getNextElement(_tmpl$11);
var _el$19 = _$getNextElement(_tmpl$12);
_$insert(_el$19, s, _el$19.firstChild);
const newLineLiteral = _el$19;
var _el$20 = _$getNextElement(_tmpl$13);
_$insert(_el$20, expr);
const trailingSpace = _el$20;
const trailingSpaceComp = _$createComponent(Comp, { children: expr });
const trailingSpaceFrag = expr;
var _el$21 = _$getNextElement(_tmpl$4);
_$insert(_el$21, expr);
const leadingSpaceElement = _el$21;
const leadingSpaceComponent = _$createComponent(Div, { get children() {
	return [" ", expr];
} });
const leadingSpaceFragment = [" ", expr];
var _el$22 = _$getNextElement(_tmpl$4);
_$insert(_el$22, expr, _el$22.firstChild);
const trailingSpaceElement = _el$22;
const trailingSpaceComponent = _$createComponent(Div, { get children() {
	return [expr, " "];
} });
const trailingSpaceFragment = [expr, " "];
const escapeAttribute = _$getNextElement(_tmpl$14);
const escapeCompAttribute = _$createComponent(Div, {
	normal: "Search…",
	title: "Search&hellip;"
});
var _el$24 = _$getNextElement(_tmpl$15);
_$insert(_el$24, expr);
const lastElementExpression = _el$24;
