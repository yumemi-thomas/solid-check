import { template as _$template } from "r-dom";
import { getNextElement as _$getNextElement } from "r-dom";
import { getNextMarker as _$getNextMarker } from "r-dom";
import { insert as _$insert } from "r-dom";
import { createComponent as _$createComponent } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<html><head><title>🔥 Blazing 🔥</title><meta charset=UTF-8><meta name=viewport content="width=device-width, initial-scale=1.0"><link rel=stylesheet href=/styles.css><!$><!/></head><body><header><h1>Welcome to the Jungle</h1></header><!$><!/><footer>The Bottom`);
var _tmpl$2 = /* @__PURE__ */ _$template(`<head><title>🔥 Blazing 🔥</title><meta charset=UTF-8><meta name=viewport content="width=device-width, initial-scale=1.0"><link rel=stylesheet href=/styles.css><!$><!/>`);
var _tmpl$3 = /* @__PURE__ */ _$template(`<body><header><h1>Welcome to the Jungle</h1></header><!$><!/><footer>The Bottom`);
var _tmpl$4 = /* @__PURE__ */ _$template(`<html><!$><!/><!$><!/>`);
const template = (() => {
	var _el$ = _$getNextElement(_tmpl$);
	var _el$2 = _el$.firstChild;
	var [_el$3, _el$4] = _$getNextMarker(_el$2.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling);
	_$insert(_el$2, _$createComponent(Assets, {}), _el$3, _el$4);
	var _el$5 = _el$.firstChild.nextSibling;
	var [_el$6, _el$7] = _$getNextMarker(_el$5.firstChild.nextSibling.nextSibling);
	_$insert(_el$5, _$createComponent(App, {}), _el$6, _el$7);
	return _el$;
})();
const templateHead = (() => {
	var _el$8 = _$getNextElement(_tmpl$2);
	var [_el$9, _el$10] = _$getNextMarker(_el$8.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling);
	_$insert(_el$8, _$createComponent(Assets, {}), _el$9, _el$10);
	return _el$8;
})();
const templateBody = (() => {
	var _el$11 = _$getNextElement(_tmpl$3);
	var [_el$12, _el$13] = _$getNextMarker(_el$11.firstChild.nextSibling.nextSibling);
	_$insert(_el$11, _$createComponent(App, {}), _el$12, _el$13);
	return _el$11;
})();
const templateEmptied = (() => {
	var _el$14 = _$getNextElement(_tmpl$4);
	var [_el$15, _el$16] = _$getNextMarker(_el$14.firstChild.nextSibling);
	_$insert(_el$14, _$createComponent(Head, {}), _el$15, _el$16);
	var [_el$17, _el$18] = _$getNextMarker(_el$14.firstChild.nextSibling.nextSibling.nextSibling);
	_$insert(_el$14, _$createComponent(Body, {}), _el$17, _el$18);
	return _el$14;
})();
