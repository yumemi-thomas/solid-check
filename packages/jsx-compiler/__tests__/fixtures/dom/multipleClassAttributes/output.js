import { template as _$template } from "r-dom";
import { className as _$className } from "r-dom";
import { effect as _$effect } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<div class=b>static static`);
var _tmpl$2 = /* @__PURE__ */ _$template(`<div>static + dynamic`);
var _tmpl$3 = /* @__PURE__ */ _$template(`<div>two dynamic`);
var _tmpl$4 = /* @__PURE__ */ _$template(`<div>mixed`);
var _tmpl$5 = /* @__PURE__ */ _$template(`<div class=c>three statics`);
// Multiple class= attributes on a DOM element should be combined into
// a single class attribute/expression.
const dynamicClass = () => "dyn";
const flag = true;
const t1 = _tmpl$();
var _el$2 = _tmpl$2();
_$effect(() => {
	return dynamicClass();
}, (_v$, _$p) => {
	_$className(_el$2, _v$, _$p);
});
const t2 = _el$2;
var _el$3 = _tmpl$3();
_$className(_el$3, flag ? "on" : "off");
const t3 = _el$3;
var _el$4 = _tmpl$4();
{
	_el$4.classList.toggle("active", !!flag);
	_el$4.classList.toggle("dim", !!!flag);
}
const t4 = _el$4;
const t5 = _tmpl$5();
