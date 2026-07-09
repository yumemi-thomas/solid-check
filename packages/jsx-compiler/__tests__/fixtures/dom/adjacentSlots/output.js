import { template as _$template } from "r-dom";
import { insert as _$insert } from "r-dom";
import { createComponent as _$createComponent } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<div><span>static</span><!><!>`);
var _tmpl$2 = /* @__PURE__ */ _$template(`<div><header><span>static</span><!><!>`);
var _tmpl$3 = /* @__PURE__ */ _$template(`<div><!><span>static`);
var _tmpl$4 = /* @__PURE__ */ _$template(`<div><span>static`);
// Per-slot `<!>` insertion markers × omitLastClosingTag: an element followed
// by multiple dynamic slots must keep its closing tag, or the trailing
// placeholders parse as its children and corrupt the template walk.
const trailingSlotsAfterElement = (() => {
	var _el$ = _tmpl$();
	var _el$2 = _el$.firstChild.nextSibling;
	var _el$3 = _el$.firstChild.nextSibling.nextSibling;
	_$insert(_el$, a, _el$2);
	_$insert(_el$, b, _el$3);
	return _el$;
})();
const trailingComponentAndSlot = (() => {
	var _el$4 = _tmpl$();
	var _el$5 = _el$4.firstChild.nextSibling;
	var _el$6 = _el$4.firstChild.nextSibling.nextSibling;
	_$insert(_el$4, _$createComponent(Comp, {}), _el$5);
	_$insert(_el$4, b, _el$6);
	return _el$4;
})();
const nestedParent = (() => {
	var _el$7 = _tmpl$2();
	var _el$8 = _el$7.firstChild;
	var _el$9 = _el$8.firstChild.nextSibling;
	var _el$10 = _el$8.firstChild.nextSibling.nextSibling;
	_$insert(_el$8, a, _el$9);
	_$insert(_el$8, b, _el$10);
	return _el$7;
})();
// Safe omissions that must be preserved:
const slotsBeforeElement = (() => {
	var _el$11 = _tmpl$3();
	var _el$12 = _el$11.firstChild;
	_$insert(_el$11, a, _el$12);
	_$insert(_el$11, b, _el$11.firstChild.nextSibling);
	return _el$11;
})();
const singleTrailingSlot = (() => {
	var _el$13 = _tmpl$4();
	_$insert(_el$13, a, null);
	return _el$13;
})();
