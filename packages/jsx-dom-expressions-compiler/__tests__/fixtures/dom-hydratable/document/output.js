import { template as _$template } from "r-dom";
import { getNextElement as _$getNextElement } from "r-dom";
import { insert as _$insert } from "r-dom";
import { createComponent as _$createComponent } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<html><head><title>🔥 Blazing 🔥</title><meta charset=UTF-8><meta name=viewport content="width=device-width, initial-scale=1.0"><link rel=stylesheet href=/styles.css></head><body><header><h1>Welcome to the Jungle</h1></header><footer>The Bottom`);
var _tmpl$2 = /* @__PURE__ */ _$template(`<head><title>🔥 Blazing 🔥</title><meta charset=UTF-8><meta name=viewport content="width=device-width, initial-scale=1.0"><link rel=stylesheet href=/styles.css>`);
var _tmpl$3 = /* @__PURE__ */ _$template(`<body><header><h1>Welcome to the Jungle</h1></header><footer>The Bottom`);
var _tmpl$4 = /* @__PURE__ */ _$template(`<html>`);
const template = (() => {
	var _el$ = _$getNextElement(_tmpl$);
	var _el$2 = _el$.firstChild;
	_$insert(_el$2, _$createComponent(Assets, {}));
	var _el$3 = _el$.firstChild.nextSibling;
	_$insert(_el$3, _$createComponent(App, {}));
	return _el$;
})();
const templateHead = (() => {
	var _el$4 = _$getNextElement(_tmpl$2);
	_$insert(_el$4, _$createComponent(Assets, {}));
	return _el$4;
})();
const templateBody = (() => {
	var _el$5 = _$getNextElement(_tmpl$3);
	_$insert(_el$5, _$createComponent(App, {}));
	return _el$5;
})();
const templateEmptied = (() => {
	var _el$6 = _$getNextElement(_tmpl$4);
	_$insert(_el$6, _$createComponent(Head, {}));
	_$insert(_el$6, _$createComponent(Body, {}));
	return _el$6;
})();
