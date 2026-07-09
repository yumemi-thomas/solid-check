import { template as _$template } from "r-dom";
import { getNextElement as _$getNextElement } from "r-dom";
import { createComponent as _$createComponent } from "r-dom";
import { spread as _$spread } from "r-dom";
import { setStyleProperty as _$setStyleProperty } from "r-dom";
import { className as _$className } from "r-dom";
import { effect as _$effect } from "r-dom";
import { setAttribute as _$setAttribute } from "r-dom";
import { setAttributeNS as _$setAttributeNS } from "r-dom";
import { runHydrationEvents as _$runHydrationEvents } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<svg width=400 height=180><rect stroke-width=2 x=50 y=20 rx=20 ry=20 width=150 height=150 style=fill:red;stroke:black;stroke-width:5;opacity:0.5></rect><linearGradient gradientTransform=rotate(25)><stop offset=0%>`);
var _tmpl$2 = /* @__PURE__ */ _$template(`<svg width=400 height=180><rect rx=20 ry=20 width=150 height=150 style=fill:red;stroke:black;opacity:0.5>`);
var _tmpl$3 = /* @__PURE__ */ _$template(`<svg width=400 height=180><rect>`);
var _tmpl$4 = /* @__PURE__ */ _$template(`<svg><rect x=50 y=20 width=150 height=150></svg>`, 2);
var _tmpl$5 = /* @__PURE__ */ _$template(`<svg viewBox="0 0 160 40"><a><text x=10 y=25>MDN Web Docs`);
var _tmpl$6 = /* @__PURE__ */ _$template(`<svg viewBox="0 0 160 40"><text x=10 y=25>`);
const template = _$getNextElement(_tmpl$);
const template2 = (() => {
	var _el$2 = _$getNextElement(_tmpl$2);
	var _el$3 = _el$2.firstChild;
	_$effect(() => {
		return state.name;
	}, (_v$, _$p) => {
		_$className(_el$3, _v$, _$p);
	});
	_$effect(() => {
		return state.width;
	}, (_v$) => {
		_$setAttribute(_el$3, "stroke-width", _v$);
	});
	_$effect(() => {
		return state.x;
	}, (_v$) => {
		_$setAttribute(_el$3, "x", _v$);
	});
	_$effect(() => {
		return state.y;
	}, (_v$) => {
		_$setAttribute(_el$3, "y", _v$);
	});
	_$effect(() => {
		return props.stroke;
	}, (_v$) => {
		_$setStyleProperty(_el$3, "stroke-width", _v$);
	});
	return _el$2;
})();
const template3 = (() => {
	var _el$4 = _$getNextElement(_tmpl$3);
	var _el$5 = _el$4.firstChild;
	_$spread(_el$5, props, false);
	_$runHydrationEvents();
	return _el$4;
})();
const template4 = _$getNextElement(_tmpl$4);
const template5 = _$getNextElement(_tmpl$4);
const template6 = _$createComponent(Component, { get children() {
	return _$getNextElement(_tmpl$4);
} });
const template7 = (() => {
	var _el$9 = _$getNextElement(_tmpl$5);
	var _el$10 = _el$9.firstChild;
	_$setAttributeNS(_el$10, "http://www.w3.org/1999/xlink", "xlink:href", url);
	return _el$9;
})();
const template8 = (() => {
	var _el$11 = _$getNextElement(_tmpl$6);
	var _el$12 = _el$11.firstChild;
	_el$12.textContent = text;
	return _el$11;
})();
