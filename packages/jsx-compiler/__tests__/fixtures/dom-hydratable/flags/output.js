import { template as _$template } from "r-dom";
import { getNextElement as _$getNextElement } from "r-dom";
import { getNextMarker as _$getNextMarker } from "r-dom";
import { insert as _$insert } from "r-dom";
import { createComponent as _$createComponent } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<div $ServerOnly><h1>Hello</h1><!$><!/><!$><!/><span>More Text`);
var _tmpl$2 = /* @__PURE__ */ _$template(`<div $ServerOnly>`);
var _tmpl$3 = /* @__PURE__ */ _$template(`<span $ServerOnly>`);
const template = (() => {
	var _el$ = _$getNextElement(_tmpl$);
	var [_el$2, _el$3] = _$getNextMarker(_el$.firstChild.nextSibling.nextSibling);
	_$insert(_el$, _$createComponent(Component, {}), _el$2, _el$3);
	var [_el$4, _el$5] = _$getNextMarker(_el$.firstChild.nextSibling.nextSibling.nextSibling.nextSibling);
	_$insert(_el$, () => {
		return state.interpolation;
	}, _el$4, _el$5);
	return _el$;
})();
const template2 = _$createComponent(Component, { get children() {
	return _$getNextElement(_tmpl$2);
} });
const template3 = _$createComponent(Component, { get children() {
	return [_$getNextElement(_tmpl$2), _$getNextElement(_tmpl$3)];
} });
const template4 = _$getNextElement(_tmpl$2);
