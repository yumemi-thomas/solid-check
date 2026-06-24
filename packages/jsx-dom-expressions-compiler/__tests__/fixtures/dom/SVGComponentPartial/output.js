import { template as _$template } from "r-dom";
import { effect as _$effect } from "r-dom";
import { setAttribute as _$setAttribute } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<g><circle r=5 fill=red>`);
const template = (() => {
	var _el$ = _tmpl$();
	var _el$2 = _el$.firstChild;
	_$effect(() => {
		return props.cx;
	}, (_v$) => {
		_$setAttribute(_el$2, "cx", _v$);
	});
	_$effect(() => {
		return props.cy;
	}, (_v$) => {
		_$setAttribute(_el$2, "cy", _v$);
	});
	return _el$;
})();
