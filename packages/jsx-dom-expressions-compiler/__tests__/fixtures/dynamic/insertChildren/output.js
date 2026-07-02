import { createComponent as _$createComponent2 } from "r-custom";
import { mergeProps as _$mergeProps2 } from "r-custom";
import { template as _$template } from "r-dom";
import { insert as _$insert } from "r-dom";
import { createComponent as _$createComponent } from "r-dom";
import { spread as _$spread } from "r-dom";
import { effect as _$effect } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<div>`);
var _tmpl$2 = /* @__PURE__ */ _$template(`<module>`);
var _tmpl$3 = /* @__PURE__ */ _$template(`<module>Hello`);
var _tmpl$4 = /* @__PURE__ */ _$template(`<module>Hi `);
var _tmpl$5 = /* @__PURE__ */ _$template(`<div>Test 1`);
const children = _tmpl$();
const dynamic = { children };
const template = _$createComponent2(Module, { children });
const template2 = (() => {
	var _el$2 = _tmpl$2();
	_$effect(() => {
		return children;
	}, (_v$) => {
		_el$2.children = _v$;
	});
	return _el$2;
})();
const template3 = (() => {
	var _el$3 = _tmpl$3();
	_$effect(() => {
		return children;
	}, (_v$) => {
		_el$3.children = _v$;
	});
	return _el$3;
})();
const template4 = (() => {
	var _el$4 = _tmpl$2();
	_$effect(() => {
		return children;
	}, (_v$) => {
		_el$4.children = _v$;
	});
	_$insert(_el$4, _$createComponent(Hello, {}));
	return _el$4;
})();
const template5 = (() => {
	var _el$5 = _tmpl$2();
	_$effect(() => {
		return dynamic.children;
	}, (_v$) => {
		_el$5.children = _v$;
	});
	return _el$5;
})();
const template6 = _$createComponent2(Module, { get children() {
	return dynamic.children;
} });
const template7 = (() => {
	var _el$6 = _tmpl$2();
	_$spread(_el$6, dynamic, false);
	return _el$6;
})();
const template8 = (() => {
	var _el$7 = _tmpl$3();
	_$spread(_el$7, dynamic, true);
	return _el$7;
})();
const template9 = (() => {
	var _el$8 = _tmpl$2();
	_$spread(_el$8, dynamic, true);
	_$insert(_el$8, () => {
		return dynamic.children;
	});
	return _el$8;
})();
const template10 = _$createComponent2(Module, _$mergeProps2(dynamic, { children: "Hello" }));
const template11 = (() => {
	var _el$9 = _tmpl$2();
	_$effect(() => {
		return state.children;
	}, (_v$) => {
		_el$9.children = _v$;
	});
	return _el$9;
})();
const template12 = _$createComponent2(Module, { children: state.children });
const template13 = (() => {
	var _el$10 = _tmpl$2();
	_$insert(_el$10, children);
	return _el$10;
})();
const template14 = _$createComponent2(Module, { get children() {
	return children;
} });
const template15 = (() => {
	var _el$11 = _tmpl$2();
	_$insert(_el$11, () => {
		return dynamic.children;
	});
	return _el$11;
})();
const template16 = _$createComponent2(Module, { get children() {
	return dynamic.children;
} });
const template18 = (() => {
	var _el$12 = _tmpl$4();
	_$insert(_el$12, children);
	return _el$12;
})();
const template19 = _$createComponent2(Module, { get children() {
	return ["Hi ", children];
} });
const template20 = (() => {
	var _el$13 = _tmpl$2();
	_$insert(_el$13, children);
	return _el$13;
})();
const template21 = _$createComponent2(Module, { get children() {
	return children();
} });
const template22 = (() => {
	var _el$14 = _tmpl$2();
	_$insert(_el$14, () => {
		return state.children();
	});
	return _el$14;
})();
const template23 = _$createComponent2(Module, { get children() {
	return state.children();
} });
const tiles = [];
tiles.push(_tmpl$5());
const template24 = (() => {
	var _el$16 = _tmpl$();
	_$insert(_el$16, tiles);
	return _el$16;
})();
const comma = (() => {
	var _el$17 = _tmpl$();
	_$insert(_el$17, (expression(), "static"));
	return _el$17;
})();
const double = (() => {
	var _el$18 = _tmpl$();
	_$insert(_el$18, () => {
		return children()();
	});
	return _el$18;
})();
