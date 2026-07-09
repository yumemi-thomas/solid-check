import { createComponent as _$createComponent2 } from "r-custom";
import { mergeProps as _$mergeProps2 } from "r-custom";
import { For as _$For } from "r-custom";
import { Show as _$Show } from "r-custom";
import { template as _$template } from "r-dom";
import { insert as _$insert } from "r-dom";
import { createComponent as _$createComponent } from "r-dom";
import { mergeProps as _$mergeProps } from "r-dom";
import { applyRef as _$applyRef } from "r-dom";
import { ref as _$ref } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<div>Hello `);
var _tmpl$2 = /* @__PURE__ */ _$template(`<div>`);
var _tmpl$3 = /* @__PURE__ */ _$template(`<div>From Parent`);
var _tmpl$4 = /* @__PURE__ */ _$template(`<div><!><!><!>`);
var _tmpl$5 = /* @__PURE__ */ _$template(`<div> | <!> | <!> | <!> | <!> | <!>`);
var _tmpl$6 = /* @__PURE__ */ _$template(`<div> | <!><!> | <!><!> | <!>`);
var _tmpl$7 = /* @__PURE__ */ _$template(`<div> | <!> |  |  | <!> | `);
var _tmpl$8 = /* @__PURE__ */ _$template(`<span>1`);
var _tmpl$9 = /* @__PURE__ */ _$template(`<span>2`);
var _tmpl$10 = /* @__PURE__ */ _$template(`<span>3`);
import { Show } from "somewhere";
const Child = (props) => {
	const [s, set] = createSignal();
	return [(() => {
		var _el$ = _tmpl$();
		{
			var _ref$ = props.ref;
			typeof _ref$ === "function" || Array.isArray(_ref$) ? _$ref(() => {
				return _ref$;
			}, _el$) : props.ref = _el$;
		}
		_$insert(_el$, () => {
			return props.name;
		}, null);
		return _el$;
	})(), (() => {
		var _el$2 = _tmpl$2();
		{
			var _ref$2 = set;
			typeof _ref$2 === "function" || Array.isArray(_ref$2) ? _$ref(() => {
				return _ref$2;
			}, _el$2) : set = _el$2;
		}
		_$insert(_el$2, () => {
			return props.children;
		});
		return _el$2;
	})()];
};
const template = (props) => {
	let childRef;
	const { content } = props;
	return (() => {
		var _el$3 = _tmpl$4();
		var _el$5 = _el$3.firstChild;
		var _el$7 = _el$3.firstChild.nextSibling;
		var _el$8 = _el$3.firstChild.nextSibling.nextSibling;
		_$insert(_el$3, _$createComponent(Child, _$mergeProps({ name: "John" }, props, {
			ref(r$) {
				var _ref$3 = childRef;
				typeof _ref$3 === "function" || Array.isArray(_ref$3) ? _$applyRef(_ref$3, r$) : childRef = r$;
			},
			booleanProperty: true,
			get children() {
				return _tmpl$3();
			}
		})), _el$5);
		_$insert(_el$3, _$createComponent(Child, _$mergeProps({ name: "Jason" }, () => {
			return dynamicSpread();
		}, {
			ref(r$) {
				var _ref$4 = props.ref;
				typeof _ref$4 === "function" || Array.isArray(_ref$4) ? _$applyRef(_ref$4, r$) : props.ref = r$;
			},
			get children() {
				var _el$6 = _tmpl$2();
				_$insert(_el$6, content);
				return _el$6;
			}
		})), _el$7);
		_$insert(_el$3, (() => {
			var _ref$5 = props.consumerRef();
			return _$createComponent(Context.Consumer, {
				ref(r$) {
					(typeof _ref$5 === "function" || Array.isArray(_ref$5)) && _$applyRef(_ref$5, r$);
				},
				children: (context) => context
			});
		})(), _el$8);
		return _el$3;
	})();
};
const template2 = _$createComponent2(Child, {
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
const template3 = _$createComponent2(Child, { get children() {
	return [
		_tmpl$2(),
		_tmpl$2(),
		_tmpl$2(),
		"After"
	];
} });
const [s, set] = createSignal();
const template4 = _$createComponent2(Child, {
	ref: set,
	get children() {
		return _tmpl$2();
	}
});
const template5 = _$createComponent2(Child, {
	get dynamic() {
		return state.dynamic;
	},
	get children() {
		return state.dynamic;
	}
});
// builtIns
const template6 = _$createComponent2(_$For, {
	get each() {
		return state.list;
	},
	get fallback() {
		return _$createComponent2(Loading, {});
	},
	get children() {
		return (item) => _$createComponent2(_$Show, {
			get when() {
				return state.condition;
			},
			get children() {
				return item;
			}
		});
	}
});
const template7 = _$createComponent2(Child, { get children() {
	return [_tmpl$2(), state.dynamic];
} });
const template8 = _$createComponent2(Child, { get children() {
	return [(item) => item, (item) => item];
} });
const template9 = _$createComponent2(_garbage, { children: "Hi" });
const template10 = (() => {
	var _el$14 = _tmpl$5();
	var _el$15 = _el$14.firstChild.nextSibling;
	var _el$16 = _el$14.firstChild.nextSibling.nextSibling.nextSibling;
	var _el$17 = _el$14.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling;
	var _el$18 = _el$14.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling;
	var _el$19 = _el$14.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling;
	_$insert(_el$14, _$createComponent(Link, { children: "new" }), _el$14.firstChild);
	_$insert(_el$14, _$createComponent(Link, { children: "comments" }), _el$15);
	_$insert(_el$14, _$createComponent(Link, { children: "show" }), _el$16);
	_$insert(_el$14, _$createComponent(Link, { children: "ask" }), _el$17);
	_$insert(_el$14, _$createComponent(Link, { children: "jobs" }), _el$18);
	_$insert(_el$14, _$createComponent(Link, { children: "submit" }), _el$19);
	return _el$14;
})();
const template11 = (() => {
	var _el$20 = _tmpl$6();
	var _el$21 = _el$20.firstChild.nextSibling;
	var _el$22 = _el$20.firstChild.nextSibling.nextSibling;
	var _el$23 = _el$20.firstChild.nextSibling.nextSibling.nextSibling.nextSibling;
	var _el$24 = _el$20.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling;
	var _el$25 = _el$20.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling;
	_$insert(_el$20, _$createComponent(Link, { children: "new" }), _el$20.firstChild);
	_$insert(_el$20, _$createComponent(Link, { children: "comments" }), _el$21);
	_$insert(_el$20, _$createComponent(Link, { children: "show" }), _el$22);
	_$insert(_el$20, _$createComponent(Link, { children: "ask" }), _el$23);
	_$insert(_el$20, _$createComponent(Link, { children: "jobs" }), _el$24);
	_$insert(_el$20, _$createComponent(Link, { children: "submit" }), _el$25);
	return _el$20;
})();
const template12 = (() => {
	var _el$26 = _tmpl$7();
	var _el$27 = _el$26.firstChild.nextSibling;
	var _el$28 = _el$26.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling;
	_$insert(_el$26, _$createComponent(Link, { children: "comments" }), _el$27);
	_$insert(_el$26, _$createComponent(Link, { children: "show" }), _el$28);
	return _el$26;
})();
class Template13 {
	render() {
		_$createComponent2(Component, {
			get prop() {
				return this.something;
			},
			onClick: () => this.shouldStay,
			get children() {
				return _$createComponent2(Nested, {
					get prop() {
						return this.data;
					},
					get children() {
						return this.content;
					}
				});
			}
		});
	}
}
const Template14 = _$createComponent2(Component, { get children() {
	return data();
} });
const Template15 = _$createComponent2(Component, props);
const Template16 = _$createComponent2(Component, _$mergeProps2({ something }, props));
const Template17 = _$createComponent2(Pre, { get children() {
	return [
		_tmpl$8(),
		" ",
		_tmpl$9(),
		" ",
		_tmpl$10()
	];
} });
const Template18 = _$createComponent2(Pre, { get children() {
	return [
		_tmpl$8(),
		_tmpl$9(),
		_tmpl$10()
	];
} });
const Template19 = _$createComponent2(Component, _$mergeProps2(() => {
	return s.dynamic();
}));
