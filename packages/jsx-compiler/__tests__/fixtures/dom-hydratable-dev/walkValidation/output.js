import { template as _$template } from "r-dom";
import { getNextElement as _$getNextElement } from "r-dom";
import { getFirstChild as _$getFirstChild } from "r-dom";
import { getNextSibling as _$getNextSibling } from "r-dom";
import { insert as _$insert } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<div><span>`);
var _tmpl$2 = /* @__PURE__ */ _$template(`<div><header>Title</header><main></main><footer>End`);
var _tmpl$3 = /* @__PURE__ */ _$template(`<span>Hello <b></b> world`);
var _tmpl$4 = /* @__PURE__ */ _$template(`<div><ul><li></ul><p>Static`);
const name = () => "dynamic";
const singleChild = (() => {
	var _el$ = _$getNextElement(_tmpl$);
	var _el$2 = _$getFirstChild(_el$, "span");
	_$insert(_el$2, name);
	return _el$;
})();
const siblingElements = (() => {
	var _el$3 = _$getNextElement(_tmpl$2);
	var _el$4 = _$getNextSibling(_el$3.firstChild, "main");
	_$insert(_el$4, name);
	return _el$3;
})();
const mixedTextAndElements = (() => {
	var _el$5 = _$getNextElement(_tmpl$3);
	var _el$6 = _$getNextSibling(_el$5.firstChild, "b");
	_$insert(_el$6, name);
	return _el$5;
})();
const nestedWalk = (() => {
	var _el$7 = _$getNextElement(_tmpl$4);
	var _el$8 = _$getFirstChild(_el$7, "ul");
	var _el$9 = _$getFirstChild(_el$8, "li");
	_$insert(_el$9, name);
	return _el$7;
})();
