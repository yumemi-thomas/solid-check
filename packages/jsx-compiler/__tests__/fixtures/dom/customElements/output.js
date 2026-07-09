import { template as _$template } from "r-dom";
import { getOwner as _$getOwner } from "r-dom";
import { effect as _$effect } from "r-dom";
import { setAttribute as _$setAttribute } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<my-element>`, 1);
var _tmpl$2 = /* @__PURE__ */ _$template(`<my-element><header slot=head>Title`, 1);
var _tmpl$3 = /* @__PURE__ */ _$template(`<slot name=head>`);
var _tmpl$4 = /* @__PURE__ */ _$template(`<a is=my-element>`, 1);
const template = (() => {
	var _el$ = _tmpl$();
	_$setAttribute(_el$, "some-attr", name);
	_$setAttribute(_el$, "notProp", data);
	_$setAttribute(_el$, "my-attr", data);
	_el$.someProp = data;
	_el$._$owner = _$getOwner();
	return _el$;
})();
const template2 = (() => {
	var _el$2 = _tmpl$();
	_$effect(() => {
		return state.name;
	}, (_v$) => {
		_$setAttribute(_el$2, "some-attr", _v$);
	});
	_$effect(() => {
		return state.data;
	}, (_v$) => {
		_$setAttribute(_el$2, "notProp", _v$);
	});
	_$effect(() => {
		return state.data;
	}, (_v$) => {
		_$setAttribute(_el$2, "my-attr", _v$);
	});
	_$effect(() => {
		return state.data;
	}, (_v$) => {
		_el$2.someProp = _v$;
	});
	_el$2._$owner = _$getOwner();
	return _el$2;
})();
const template3 = (() => {
	var _el$3 = _tmpl$2();
	_el$3._$owner = _$getOwner();
	return _el$3;
})();
const template4 = (() => {
	var _el$4 = _tmpl$3();
	_el$4._$owner = _$getOwner();
	return _el$4;
})();
var _el$5 = _tmpl$4();
_el$5._$owner = _$getOwner();
const template5 = _el$5;
