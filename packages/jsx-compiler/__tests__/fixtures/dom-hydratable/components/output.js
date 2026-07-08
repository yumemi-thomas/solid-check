import { template as _$template } from "r-dom";
import { getNextElement as _$getNextElement } from "r-dom";
import { getNextMarker as _$getNextMarker } from "r-dom";
import { insert as _$insert } from "r-dom";
import { scope as _$scope } from "r-dom";
import { memo as _$memo } from "r-dom";
import { createComponent as _$createComponent } from "r-dom";
import { mergeProps as _$mergeProps } from "r-dom";
import { applyRef as _$applyRef } from "r-dom";
import { ref as _$ref } from "r-dom";
import { For as _$For } from "r-dom";
import { Show as _$Show } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<div>Hello <!$><!/>`);
var _tmpl$2 = /* @__PURE__ */ _$template(`<div>`);
var _tmpl$3 = /* @__PURE__ */ _$template(`<div>From Parent`);
var _tmpl$4 = /* @__PURE__ */ _$template(`<div><!$><!/><!$><!/><!$><!/>`);
var _tmpl$5 = /* @__PURE__ */ _$template(`<div><!$><!/> | <!$><!/> | <!$><!/> | <!$><!/> | <!$><!/> | <!$><!/>`);
var _tmpl$6 = /* @__PURE__ */ _$template(`<div><!$><!/> | <!$><!/><!$><!/> | <!$><!/><!$><!/> | <!$><!/>`);
var _tmpl$7 = /* @__PURE__ */ _$template(`<div> | <!$><!/> |  |  | <!$><!/> | `);
var _tmpl$8 = /* @__PURE__ */ _$template(`<span>1`);
var _tmpl$9 = /* @__PURE__ */ _$template(`<span>2`);
var _tmpl$10 = /* @__PURE__ */ _$template(`<span>3`);
import { Show } from "somewhere";
const Child = (props) => {
	const [s, set] = createSignal();
	return [(() => {
		var _el$ = _$getNextElement(_tmpl$);
		{
			var _ref$ = props.ref;
			typeof _ref$ === "function" || Array.isArray(_ref$) ? _$ref(() => {
				return _ref$;
			}, _el$) : props.ref = _el$;
		}
		var [_el$2, _el$3] = _$getNextMarker(_el$.firstChild.nextSibling.nextSibling);
		_$insert(_el$, () => {
			return props.name;
		}, _el$2, _el$3);
		return _el$;
	})(), (() => {
		var _el$4 = _$getNextElement(_tmpl$2);
		{
			var _ref$2 = set;
			typeof _ref$2 === "function" || Array.isArray(_ref$2) ? _$ref(() => {
				return _ref$2;
			}, _el$4) : set = _el$4;
		}
		_$insert(_el$4, _$scope(() => {
			return props.children;
		}));
		return _el$4;
	})()];
};
const template = (props) => {
	let childRef;
	const { content } = props;
	var _el$5 = _$getNextElement(_tmpl$4);
	var [_el$7, _el$8] = _$getNextMarker(_el$5.firstChild.nextSibling);
	_$insert(_el$5, _$createComponent(Child, _$mergeProps({ name: "John" }, props, {
		ref(r$) {
			var _ref$3 = childRef;
			typeof _ref$3 === "function" || Array.isArray(_ref$3) ? _$applyRef(_ref$3, r$) : childRef = r$;
		},
		booleanProperty: true,
		get children() {
			return _$getNextElement(_tmpl$3);
		}
	})), _el$7, _el$8);
	var [_el$10, _el$11] = _$getNextMarker(_el$5.firstChild.nextSibling.nextSibling.nextSibling);
	_$insert(_el$5, _$createComponent(Child, _$mergeProps({ name: "Jason" }, () => {
		return dynamicSpread();
	}, {
		ref(r$) {
			var _ref$4 = props.ref;
			typeof _ref$4 === "function" || Array.isArray(_ref$4) ? _$applyRef(_ref$4, r$) : props.ref = r$;
		},
		get children() {
			var _el$9 = _$getNextElement(_tmpl$2);
			_$insert(_el$9, content);
			return _el$9;
		}
	})), _el$10, _el$11);
	var [_el$12, _el$13] = _$getNextMarker(_el$5.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling);
	_$insert(_el$5, (() => {
		var _ref$5 = props.consumerRef();
		return _$createComponent(Context.Consumer, {
			ref(r$) {
				(typeof _ref$5 === "function" || Array.isArray(_ref$5)) && _$applyRef(_ref$5, r$);
			},
			children: (context) => context
		});
	})(), _el$12, _el$13);
	return _el$5;
};
const template2 = _$createComponent(Child, {
	name: "Jake",
	get dynamic() {
		return state.data;
	},
	stale: state.data,
	handleClick: clickHandler,
	get "hyphen-ated"() {
		return state.data;
	},
	ref: (el) => e = el
});
const template3 = _$createComponent(Child, { get children() {
	return [
		_$getNextElement(_tmpl$2),
		_$getNextElement(_tmpl$2),
		_$getNextElement(_tmpl$2),
		"After"
	];
} });
const [s, set] = createSignal();
const template4 = _$createComponent(Child, {
	ref: set,
	get children() {
		return _$getNextElement(_tmpl$2);
	}
});
const template5 = _$createComponent(Child, {
	get dynamic() {
		return state.dynamic;
	},
	get children() {
		return state.dynamic;
	}
});
// builtIns
const template6 = _$createComponent(_$For, {
	get each() {
		return state.list;
	},
	get fallback() {
		return _$createComponent(Loading, {});
	},
	children: (item) => _$createComponent(_$Show, {
		get when() {
			return state.condition;
		},
		children: item
	})
});
const template7 = _$createComponent(Child, { get children() {
	return [_$getNextElement(_tmpl$2), _$memo(() => {
		return state.dynamic;
	})];
} });
const template8 = _$createComponent(Child, { get children() {
	return [(item) => item, (item) => item];
} });
const template9 = _$createComponent(_garbage, { children: "Hi" });
const template10 = (() => {
	var _el$19 = _$getNextElement(_tmpl$5);
	var [_el$20, _el$21] = _$getNextMarker(_el$19.firstChild.nextSibling);
	_$insert(_el$19, _$createComponent(Link, { children: "new" }), _el$20, _el$21);
	var [_el$22, _el$23] = _$getNextMarker(_el$19.firstChild.nextSibling.nextSibling.nextSibling.nextSibling);
	_$insert(_el$19, _$createComponent(Link, { children: "comments" }), _el$22, _el$23);
	var [_el$24, _el$25] = _$getNextMarker(_el$19.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling);
	_$insert(_el$19, _$createComponent(Link, { children: "show" }), _el$24, _el$25);
	var [_el$26, _el$27] = _$getNextMarker(_el$19.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling);
	_$insert(_el$19, _$createComponent(Link, { children: "ask" }), _el$26, _el$27);
	var [_el$28, _el$29] = _$getNextMarker(_el$19.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling);
	_$insert(_el$19, _$createComponent(Link, { children: "jobs" }), _el$28, _el$29);
	var [_el$30, _el$31] = _$getNextMarker(_el$19.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling);
	_$insert(_el$19, _$createComponent(Link, { children: "submit" }), _el$30, _el$31);
	return _el$19;
})();
const template11 = (() => {
	var _el$32 = _$getNextElement(_tmpl$6);
	var [_el$33, _el$34] = _$getNextMarker(_el$32.firstChild.nextSibling);
	_$insert(_el$32, _$createComponent(Link, { children: "new" }), _el$33, _el$34);
	var [_el$35, _el$36] = _$getNextMarker(_el$32.firstChild.nextSibling.nextSibling.nextSibling.nextSibling);
	_$insert(_el$32, _$createComponent(Link, { children: "comments" }), _el$35, _el$36);
	var [_el$37, _el$38] = _$getNextMarker(_el$32.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling);
	_$insert(_el$32, _$createComponent(Link, { children: "show" }), _el$37, _el$38);
	var [_el$39, _el$40] = _$getNextMarker(_el$32.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling);
	_$insert(_el$32, _$createComponent(Link, { children: "ask" }), _el$39, _el$40);
	var [_el$41, _el$42] = _$getNextMarker(_el$32.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling);
	_$insert(_el$32, _$createComponent(Link, { children: "jobs" }), _el$41, _el$42);
	var [_el$43, _el$44] = _$getNextMarker(_el$32.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling);
	_$insert(_el$32, _$createComponent(Link, { children: "submit" }), _el$43, _el$44);
	return _el$32;
})();
const template12 = (() => {
	var _el$45 = _$getNextElement(_tmpl$7);
	var [_el$46, _el$47] = _$getNextMarker(_el$45.firstChild.nextSibling.nextSibling);
	_$insert(_el$45, _$createComponent(Link, { children: "comments" }), _el$46, _el$47);
	var [_el$48, _el$49] = _$getNextMarker(_el$45.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling);
	_$insert(_el$45, _$createComponent(Link, { children: "show" }), _el$48, _el$49);
	return _el$45;
})();
class Template13 {
	render() {
		const _self$ = this;
		_$createComponent(Component, {
			get prop() {
				return _self$.something;
			},
			onClick: () => _self$.shouldStay,
			get children() {
				return _$createComponent(Nested, {
					get prop() {
						return _self$.data;
					},
					get children() {
						return _self$.content;
					}
				});
			}
		});
	}
}
const Template14 = _$createComponent(Component, { get children() {
	return data();
} });
const Template15 = _$createComponent(Component, props);
const Template16 = _$createComponent(Component, _$mergeProps({ something }, props));
const Template17 = _$createComponent(Pre, { get children() {
	return [
		_$getNextElement(_tmpl$8),
		" ",
		_$getNextElement(_tmpl$9),
		" ",
		_$getNextElement(_tmpl$10)
	];
} });
const Template18 = _$createComponent(Pre, { get children() {
	return [
		_$getNextElement(_tmpl$8),
		_$getNextElement(_tmpl$9),
		_$getNextElement(_tmpl$10)
	];
} });
const Template19 = _$createComponent(Component, _$mergeProps(() => {
	return s.dynamic();
}));
const Template20 = _$createComponent(Component, { get class() {
	return prop.red ? "red" : "green";
} });
const template21 = _$createComponent(Component, { get [key()]() {
	return props.value;
} });
const template22 = _$createComponent(Component, { passObject: { ...a } });
const template23 = _$createComponent(Component, {
	disabled: "t" in test,
	get children() {
		return "t" in test && "true";
	}
});
const template24 = _$createComponent(Component, { get children() {
	return state.dynamic;
} });
const template25 = _$createComponent(Component, { get children() {
	return _$getNextElement(_tmpl$2);
} });
