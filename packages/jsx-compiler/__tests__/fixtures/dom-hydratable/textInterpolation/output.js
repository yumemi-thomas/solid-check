import { template as _$template } from "r-dom";
import { getNextElement as _$getNextElement } from "r-dom";
import { getNextMarker as _$getNextMarker } from "r-dom";
import { insert as _$insert } from "r-dom";
import { scope as _$scope } from "r-dom";
import { createComponent as _$createComponent } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<span>Hello `);
var _tmpl$2 = /* @__PURE__ */ _$template(`<span> John`);
var _tmpl$3 = /* @__PURE__ */ _$template(`<span>Hello John`);
var _tmpl$4 = /* @__PURE__ */ _$template(`<span>Hello <!$><!/>`);
var _tmpl$5 = /* @__PURE__ */ _$template(`<span><!$><!/> John`);
var _tmpl$6 = /* @__PURE__ */ _$template(`<span><!$><!/> <!$><!/>`);
var _tmpl$7 = /* @__PURE__ */ _$template(`<span> <!> <!> `);
var _tmpl$8 = /* @__PURE__ */ _$template(`<span> <!> `);
var _tmpl$9 = /* @__PURE__ */ _$template(`<span>Hello`);
var _tmpl$10 = /* @__PURE__ */ _$template(`<span>&nbsp;&lt;Hi&gt;&nbsp;`);
var _tmpl$11 = /* @__PURE__ */ _$template(`<span>Hi&lt;script>alert();&lt;/script>`);
var _tmpl$12 = /* @__PURE__ */ _$template(`<span>Hello World!`);
var _tmpl$13 = /* @__PURE__ */ _$template(`<span>4 + 5 = 9`);
var _tmpl$14 = /* @__PURE__ */ _$template(`<div><!$><!/>
d`);
var _tmpl$15 = /* @__PURE__ */ _$template(`<div>`);
var _tmpl$16 = /* @__PURE__ */ _$template(`<span> <!$><!/>`);
var _tmpl$17 = /* @__PURE__ */ _$template(`<span><!$><!/> `);
var _tmpl$18 = /* @__PURE__ */ _$template(`<div normal=Search… title=Search&amp;hellip;>`);
var _tmpl$19 = /* @__PURE__ */ _$template(`<div><div><!$><!/>`);
const trailing = _$getNextElement(_tmpl$);
const leading = _$getNextElement(_tmpl$2);
/* prettier-ignore */
const extraSpaces = _$getNextElement(_tmpl$3);
var _el$4 = _$getNextElement(_tmpl$4);
var [_el$5, _el$6] = _$getNextMarker(_el$4.firstChild.nextSibling.nextSibling);
_$insert(_el$4, name, _el$5, _el$6);
const trailingExpr = _el$4;
var _el$7 = _$getNextElement(_tmpl$5);
var [_el$8, _el$9] = _$getNextMarker(_el$7.firstChild.nextSibling);
_$insert(_el$7, greeting, _el$8, _el$9);
const leadingExpr = _el$7;
var _el$10 = _$getNextElement(_tmpl$6);
var [_el$11, _el$12] = _$getNextMarker(_el$10.firstChild.nextSibling);
_$insert(_el$10, greeting, _el$11, _el$12);
var [_el$13, _el$14] = _$getNextMarker(_el$10.firstChild.nextSibling.nextSibling.nextSibling.nextSibling);
_$insert(_el$10, name, _el$13, _el$14);
/* prettier-ignore */
const multiExpr = _el$10;
var _el$15 = _$getNextElement(_tmpl$7);
var _el$16 = _el$15.firstChild.nextSibling;
_$insert(_el$15, greeting, _el$16);
var _el$17 = _el$15.firstChild.nextSibling.nextSibling.nextSibling;
_$insert(_el$15, name, _el$17);
/* prettier-ignore */
const multiExprSpaced = _el$15;
var _el$18 = _$getNextElement(_tmpl$8);
var _el$19 = _el$18.firstChild.nextSibling;
_$insert(_el$18, greeting, _el$19);
_$insert(_el$18, name, _el$19);
/* prettier-ignore */
const multiExprTogether = _el$18;
/* prettier-ignore */
const multiLine = _$getNextElement(_tmpl$9);
/* prettier-ignore */
const multiLineTrailingSpace = _$getNextElement(_tmpl$3);
/* prettier-ignore */
const multiLineNoTrailingSpace = _$getNextElement(_tmpl$3);
/* prettier-ignore */
const escape = _$getNextElement(_tmpl$10);
/* prettier-ignore */
const escape2 = _$createComponent(Comp, { children: "\xA0<Hi>\xA0" });
/* prettier-ignore */
const escape3 = "\xA0<Hi>\xA0";
const injection = _$getNextElement(_tmpl$11);
let value = "World";
const evaluated = _$getNextElement(_tmpl$12);
let number = 4 + 5;
const evaluatedNonString = _$getNextElement(_tmpl$13);
var _el$27 = _$getNextElement(_tmpl$14);
var [_el$28, _el$29] = _$getNextMarker(_el$27.firstChild.nextSibling);
_$insert(_el$27, s, _el$28, _el$29);
const newLineLiteral = _el$27;
var _el$30 = _$getNextElement(_tmpl$15);
_$insert(_el$30, expr);
const trailingSpace = _el$30;
const trailingSpaceComp = _$createComponent(Comp, { children: expr });
const trailingSpaceFrag = expr;
var _el$31 = _$getNextElement(_tmpl$16);
var [_el$32, _el$33] = _$getNextMarker(_el$31.firstChild.nextSibling.nextSibling);
_$insert(_el$31, expr, _el$32, _el$33);
const leadingSpaceElement = _el$31;
const leadingSpaceComponent = _$createComponent(Div, { get children() {
	return [" ", expr];
} });
const leadingSpaceFragment = [" ", expr];
var _el$34 = _$getNextElement(_tmpl$17);
var [_el$35, _el$36] = _$getNextMarker(_el$34.firstChild.nextSibling);
_$insert(_el$34, expr, _el$35, _el$36);
const trailingSpaceElement = _el$34;
const trailingSpaceComponent = _$createComponent(Div, { get children() {
	return [expr, " "];
} });
const trailingSpaceFragment = [expr, " "];
const escapeAttribute = _$getNextElement(_tmpl$18);
const escapeCompAttribute = _$createComponent(Div, {
	normal: "Search…",
	title: "Search&hellip;"
});
var _el$38 = _$getNextElement(_tmpl$19);
var [_el$39, _el$40] = _$getNextMarker(_el$38.firstChild.nextSibling.nextSibling);
_$insert(_el$38, _$scope(() => {
	return expr();
}), _el$39, _el$40);
const lastElementExpression = _el$38;
