import { template as _$template } from "r-dom";
import { insert as _$insert } from "r-dom";
import { memo as _$memo } from "r-dom";
import { createComponent as _$createComponent } from "r-dom";
import { spread as _$spread } from "r-dom";
import { mergeProps as _$mergeProps } from "r-dom";
import { applyRef as _$applyRef } from "r-dom";
import { ref as _$ref } from "r-dom";
import { For as _$For } from "r-dom";
import { Show as _$Show } from "r-dom";
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
import { Show, binding } from "somewhere";
function refFn() {}
const refConst = null;
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
	var _el$3 = _tmpl$4();
	var _el$5 = _el$3.firstChild;
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
	var _el$7 = _el$3.firstChild.nextSibling;
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
	var _el$8 = _el$3.firstChild.nextSibling.nextSibling;
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
		_tmpl$2(),
		_tmpl$2(),
		_tmpl$2(),
		"After"
	];
} });
const [s, set] = createSignal();
const template4 = _$createComponent(Child, {
	ref: set,
	get children() {
		return _tmpl$2();
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
	return [_tmpl$2(), _$memo(() => {
		return state.dynamic;
	})];
} });
const template8 = _$createComponent(Child, { get children() {
	return [(item) => item, (item) => item];
} });
const template9 = _$createComponent(_garbage, { children: "Hi" });
const template10 = (() => {
	var _el$14 = _tmpl$5();
	_$insert(_el$14, _$createComponent(Link, { children: "new" }), _el$14.firstChild);
	var _el$15 = _el$14.firstChild.nextSibling;
	_$insert(_el$14, _$createComponent(Link, { children: "comments" }), _el$15);
	var _el$16 = _el$14.firstChild.nextSibling.nextSibling.nextSibling;
	_$insert(_el$14, _$createComponent(Link, { children: "show" }), _el$16);
	var _el$17 = _el$14.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling;
	_$insert(_el$14, _$createComponent(Link, { children: "ask" }), _el$17);
	var _el$18 = _el$14.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling;
	_$insert(_el$14, _$createComponent(Link, { children: "jobs" }), _el$18);
	var _el$19 = _el$14.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling;
	_$insert(_el$14, _$createComponent(Link, { children: "submit" }), _el$19);
	return _el$14;
})();
const template11 = (() => {
	var _el$20 = _tmpl$6();
	_$insert(_el$20, _$createComponent(Link, { children: "new" }), _el$20.firstChild);
	var _el$21 = _el$20.firstChild.nextSibling;
	_$insert(_el$20, _$createComponent(Link, { children: "comments" }), _el$21);
	var _el$22 = _el$20.firstChild.nextSibling.nextSibling;
	_$insert(_el$20, _$createComponent(Link, { children: "show" }), _el$22);
	var _el$23 = _el$20.firstChild.nextSibling.nextSibling.nextSibling.nextSibling;
	_$insert(_el$20, _$createComponent(Link, { children: "ask" }), _el$23);
	var _el$24 = _el$20.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling;
	_$insert(_el$20, _$createComponent(Link, { children: "jobs" }), _el$24);
	var _el$25 = _el$20.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling;
	_$insert(_el$20, _$createComponent(Link, { children: "submit" }), _el$25);
	return _el$20;
})();
const template12 = (() => {
	var _el$26 = _tmpl$7();
	var _el$27 = _el$26.firstChild.nextSibling;
	_$insert(_el$26, _$createComponent(Link, { children: "comments" }), _el$27);
	var _el$28 = _el$26.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling;
	_$insert(_el$26, _$createComponent(Link, { children: "show" }), _el$28);
	return _el$26;
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
		_tmpl$8(),
		" ",
		_tmpl$9(),
		" ",
		_tmpl$10()
	];
} });
const Template18 = _$createComponent(Pre, { get children() {
	return [
		_tmpl$8(),
		_tmpl$9(),
		_tmpl$10()
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
	return _tmpl$2();
} });
const template26 = [_$createComponent(Component, { get when() {
	return (() => {
		const foo = test();
		if ("t" in foo) {
			return foo;
		}
	})();
} }), _$createComponent(Component, { get when() {
	return ((val = 123) => {
		return val * 2;
	})();
} })];
const template27 = _$createComponent(Component, { get when() {
	return (() => prop.red ? "red" : "green")();
} });
class Template28 {
	render() {
		const _self$2 = this;
		return _$createComponent(Component, { get when() {
			return (() => {
				const foo = _self$2.value;
				if ("key" in foo) {
					return foo;
				}
			})();
		} });
	}
}
class Template29 extends ParentComponent {
	constructor() {
		super();
		const _self$3 = this;
		_$createComponent(_self$3.component, { get method() {
			return _self$3.method;
		} });
	}
	get get() {
		const _self$4 = this;
		_$createComponent(_self$4.component, { get method() {
			return _self$4.method;
		} });
	}
	set set(v) {
		const _self$5 = this;
		_$createComponent(_self$5.component, { get method() {
			return _self$5.method;
		} });
	}
	method() {
		const _self$6 = this;
		_$createComponent(_self$6.component, { get method() {
			return _self$6.method;
		} });
	}
	field = (() => {
		const _self$7 = this;
		return _$createComponent(_self$7.component, {
			get method() {
				return _self$7.method;
			},
			get comp() {
				return _$createComponent(_self$7.another, {});
			}
		});
	})();
	fieldArrow = () => {
		const _self$8 = this;
		return _$createComponent(_self$8.component, { get method() {
			return _self$8.method;
		} });
	};
	fieldFunction = function() {
		const _self$9 = this;
		_$createComponent(_self$9.component, { get method() {
			return _self$9.method;
		} });
	};
}
const template30 = _$createComponent(Comp, { ref: binding });
const template31 = _$createComponent(Comp, { ref(r$) {
	var _ref$6 = binding.prop;
	typeof _ref$6 === "function" || Array.isArray(_ref$6) ? _$applyRef(_ref$6, r$) : binding.prop = r$;
} });
const template32 = _$createComponent(Comp, { ref(r$) {
	var _ref$7 = refFn;
	typeof _ref$7 === "function" || Array.isArray(_ref$7) ? _$applyRef(_ref$7, r$) : refFn = r$;
} });
const template33 = _$createComponent(Comp, { ref: refConst });
const template34 = _$createComponent(Comp, { ref(r$) {
	var _ref$8 = refUnknown;
	typeof _ref$8 === "function" || Array.isArray(_ref$8) ? _$applyRef(_ref$8, r$) : refUnknown = r$;
} });
const template35 = _$createComponent(Comp, { ref(r$) {
	var _ref$9 = binding?.prop;
	typeof _ref$9 === "function" || Array.isArray(_ref$9) ? _$applyRef(_ref$9, r$) : !!binding && (binding.prop = r$);
} });
const template36 = _$createComponent(Comp, {});
const template37 = _$createComponent(Comp, {});
function MyComponent(props) {
	let el;
	const others = omit(props, "children");
	var _el$36 = _tmpl$2();
	_$spread(_el$36, _$mergeProps({ ref: el }, others), true);
	_$insert(_el$36, () => {
		return props.children;
	});
	return _el$36;
}
