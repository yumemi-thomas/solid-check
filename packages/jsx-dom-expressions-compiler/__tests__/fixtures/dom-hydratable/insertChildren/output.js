import { template as _$template } from "r-dom";
import { getNextElement as _$getNextElement } from "r-dom";
import { insert as _$insert } from "r-dom";
import { createComponent as _$createComponent } from "r-dom";
import { spread as _$spread } from "r-dom";
import { mergeProps as _$mergeProps } from "r-dom";
import { effect as _$effect } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<div>`);
var _tmpl$2 = /* @__PURE__ */ _$template(`<module>`);
var _tmpl$3 = /* @__PURE__ */ _$template(`<module>Hello`);
var _tmpl$4 = /* @__PURE__ */ _$template(`<module>Hi `);
var _tmpl$5 = /* @__PURE__ */ _$template(`<module>Hi`);
var _tmpl$6 = /* @__PURE__ */ _$template(`<div>Test 1`);
const children = _$getNextElement(_tmpl$);
const dynamic = { children };
const template = _$createComponent(Module, { children });
var _el$2 = _$getNextElement(_tmpl$2);
_$effect(() => {
	return children;
}, (_v$) => {
	_el$2.children = _v$;
});
const template2 = _el$2;
var _el$3 = _$getNextElement(_tmpl$3);
_$effect(() => {
	return children;
}, (_v$) => {
	_el$3.children = _v$;
});
const template3 = _el$3;
const template4 = (() => {
	var _el$4 = _$getNextElement(_tmpl$2);
	_$effect(() => {
		return children;
	}, (_v$) => {
		_el$4.children = _v$;
	});
	_$insert(_el$4, _$createComponent(Hello, {}));
	return _el$4;
})();
var _el$5 = _$getNextElement(_tmpl$2);
_$effect(() => {
	return dynamic.children;
}, (_v$) => {
	_el$5.children = _v$;
});
const template5 = _el$5;
const template6 = _$createComponent(Module, { get children() {
	return dynamic.children;
} });
var _el$6 = _$getNextElement(_tmpl$2);
_$spread(_el$6, dynamic, false);
const template7 = _el$6;
var _el$7 = _$getNextElement(_tmpl$3);
_$spread(_el$7, dynamic, true);
const template8 = _el$7;
var _el$8 = _$getNextElement(_tmpl$2);
_$spread(_el$8, dynamic, true);
_$insert(_el$8, () => {
	return dynamic.children;
});
const template9 = _el$8;
const template10 = _$createComponent(Module, _$mergeProps(dynamic, { children: "Hello" }));
var _el$9 = _$getNextElement(_tmpl$2);
_$effect(() => {
	return state.children;
}, (_v$) => {
	_el$9.children = _v$;
});
const template11 = _el$9;
const template12 = _$createComponent(Module, { children: state.children });
var _el$10 = _$getNextElement(_tmpl$2);
_$insert(_el$10, children);
const template13 = _el$10;
const template14 = _$createComponent(Module, { children });
var _el$11 = _$getNextElement(_tmpl$2);
_$insert(_el$11, () => {
	return dynamic.children;
});
const template15 = _el$11;
const template16 = _$createComponent(Module, { get children() {
	return dynamic.children;
} });
var _el$12 = _$getNextElement(_tmpl$4);
_$insert(_el$12, children);
const template18 = _el$12;
const template19 = _$createComponent(Module, { get children() {
	return ["Hi ", children];
} });
var _el$13 = _$getNextElement(_tmpl$2);
_$insert(_el$13, children);
const template20 = _el$13;
const template21 = _$createComponent(Module, { get children() {
	return children();
} });
var _el$14 = _$getNextElement(_tmpl$2);
_$insert(_el$14, () => {
	return state.children();
});
const template22 = _el$14;
const template23 = _$createComponent(Module, { get children() {
	return state.children();
} });
var _el$15 = _$getNextElement(_tmpl$5);
_$spread(_el$15, dynamic, true);
_$insert(_el$15, () => {
	return dynamic.children;
});
const template24 = _el$15;
const tiles = [];
tiles.push(_$getNextElement(_tmpl$6));
var _el$17 = _$getNextElement(_tmpl$);
_$insert(_el$17, tiles);
const template25 = _el$17;
var _el$18 = _$getNextElement(_tmpl$);
_$insert(_el$18, (expression(), "static"));
const comma = _el$18;
var _el$19 = _$getNextElement(_tmpl$);
_$insert(_el$19, () => {
	return children()();
});
const double = _el$19;
