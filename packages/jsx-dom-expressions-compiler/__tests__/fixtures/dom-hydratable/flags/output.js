import { template as _$template } from "r-dom";
import { getNextElement as _$getNextElement } from "r-dom";
import { insert as _$insert } from "r-dom";
import { createComponent as _$createComponent } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<div $ServerOnly><h1>Hello</h1><span>More Text`);
var _tmpl$2 = /* @__PURE__ */ _$template(`<div $ServerOnly>`);
var _tmpl$3 = /* @__PURE__ */ _$template(`<span $ServerOnly>`);
const template = (() => {
	var _el$ = _$getNextElement(_tmpl$);
	_$insert(_el$, _$createComponent(Component, {}));
	_$insert(_el$, () => {
		return state.interpolation;
	}, _el$.firstChild);
	return _el$;
})();
const template2 = _$createComponent(Component, { get children() {
	return _$getNextElement(_tmpl$2);
} });
const template3 = _$createComponent(Component, { get children() {
	return [_$getNextElement(_tmpl$2), _$getNextElement(_tmpl$3)];
} });
const template4 = _$getNextElement(_tmpl$2);
