import { createTextNode as _$createTextNode } from "r-custom";
import { createComponent as _$createComponent } from "r-custom";
import { mergeProps as _$mergeProps } from "r-custom";
import { insert as _$insert } from "r-custom";
import { insertNode as _$insertNode } from "r-custom";
import { setProp as _$setProp } from "r-custom";
import { createElement as _$createElement } from "r-custom";
import { Show, binding } from "somewhere";
function refFn() {}
const refConst = null;
const Child = (props) => [(() => {
	var _el$ = _$createElement("div");
	_$setProp(_el$, "ref", props.ref);
	_$insertNode(_el$, _$createTextNode("Hello "));
	_$insert(_el$, props.name);
	return _el$;
})(), (() => {
	var _el$2 = _$createElement("div");
	_$insert(_el$2, props.children);
	return _el$2;
})()];
const template = (props) => {
	let childRef;
	const { content } = props;
	return (() => {
		var _el$3 = _$createElement("div");
		_$insertNode(_el$3, _$createComponent(Child, _$mergeProps({ name: "John" }, props, {
			ref: childRef,
			booleanProperty: true,
			get children() {
				return (() => {
					var _el$4 = _$createElement("div");
					_$insertNode(_el$4, _$createTextNode("From Parent"));
					return _el$4;
				})();
			}
		})));
		_$insertNode(_el$3, _$createComponent(Child, _$mergeProps({ name: "Jason" }, () => {
			return dynamicSpread();
		}, {
			ref: props.ref,
			get children() {
				return (() => {
					var _el$5 = _$createElement("div");
					_$insert(_el$5, content);
					return _el$5;
				})();
			}
		})));
		_$insertNode(_el$3, _$createComponent(Context.Consumer, {
			ref: props.consumerRef(),
			get children() {
				return (context) => context;
			}
		}));
		return _el$3;
	})();
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
		(() => {
			var _el$6 = _$createElement("div");
			return _el$6;
		})(),
		(() => {
			var _el$7 = _$createElement("div");
			return _el$7;
		})(),
		(() => {
			var _el$8 = _$createElement("div");
			return _el$8;
		})(),
		"After"
	];
} });
const template4 = _$createComponent(Child, { get children() {
	return (() => {
		var _el$9 = _$createElement("div");
		return _el$9;
	})();
} });
const template5 = _$createComponent(Child, {
	get dynamic() {
		return state.dynamic;
	},
	get children() {
		return state.dynamic;
	}
});
// builtIns
const template6 = _$createComponent(For, {
	get each() {
		return state.list;
	},
	get fallback() {
		return _$createComponent(Loading, {});
	},
	get children() {
		return (item) => _$createComponent(Show, {
			get when() {
				return state.condition;
			},
			get children() {
				return item;
			}
		});
	}
});
const template7 = _$createComponent(Child, { get children() {
	return [(() => {
		var _el$10 = _$createElement("div");
		return _el$10;
	})(), state.dynamic];
} });
const template8 = _$createComponent(Child, { get children() {
	return [(item) => item, (item) => item];
} });
const template9 = _$createComponent(_garbage, { children: "Hi" });
var _el$11 = _$createElement("div");
_$insertNode(_el$11, _$createComponent(Link, { children: "new" }));
_$insertNode(_el$11, _$createTextNode(" | "));
_$insertNode(_el$11, _$createComponent(Link, { children: "comments" }));
_$insertNode(_el$11, _$createTextNode(" | "));
_$insertNode(_el$11, _$createComponent(Link, { children: "show" }));
_$insertNode(_el$11, _$createTextNode(" | "));
_$insertNode(_el$11, _$createComponent(Link, { children: "ask" }));
_$insertNode(_el$11, _$createTextNode(" | "));
_$insertNode(_el$11, _$createComponent(Link, { children: "jobs" }));
_$insertNode(_el$11, _$createTextNode(" | "));
_$insertNode(_el$11, _$createComponent(Link, { children: "submit" }));
const template10 = _el$11;
var _el$12 = _$createElement("div");
_$insertNode(_el$12, _$createComponent(Link, { children: "new" }));
_$insertNode(_el$12, _$createTextNode(" | "));
_$insertNode(_el$12, _$createComponent(Link, { children: "comments" }));
_$insertNode(_el$12, _$createComponent(Link, { children: "show" }));
_$insertNode(_el$12, _$createTextNode(" | "));
_$insertNode(_el$12, _$createComponent(Link, { children: "ask" }));
_$insertNode(_el$12, _$createComponent(Link, { children: "jobs" }));
_$insertNode(_el$12, _$createTextNode(" | "));
_$insertNode(_el$12, _$createComponent(Link, { children: "submit" }));
const template11 = _el$12;
var _el$13 = _$createElement("div");
_$insertNode(_el$13, _$createTextNode(" | "));
_$insertNode(_el$13, _$createComponent(Link, { children: "comments" }));
_$insertNode(_el$13, _$createTextNode(" | "));
_$insertNode(_el$13, _$createTextNode(" | "));
_$insertNode(_el$13, _$createTextNode(" | "));
_$insertNode(_el$13, _$createComponent(Link, { children: "show" }));
_$insertNode(_el$13, _$createTextNode(" | "));
const template12 = _el$13;
class Template13 {
	render() {
		_$createComponent(Component, {
			get prop() {
				return this.something;
			},
			onClick: () => this.shouldStay,
			get children() {
				return _$createComponent(Nested, {
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
const Template14 = _$createComponent(Component, { get children() {
	return data();
} });
const Template15 = _$createComponent(Component, props);
const Template16 = _$createComponent(Component, _$mergeProps({ something }, props));
const Template17 = _$createComponent(Pre, { get children() {
	return [
		(() => {
			var _el$14 = _$createElement("span");
			_$insertNode(_el$14, _$createTextNode("1"));
			return _el$14;
		})(),
		" ",
		(() => {
			var _el$15 = _$createElement("span");
			_$insertNode(_el$15, _$createTextNode("2"));
			return _el$15;
		})(),
		" ",
		(() => {
			var _el$16 = _$createElement("span");
			_$insertNode(_el$16, _$createTextNode("3"));
			return _el$16;
		})()
	];
} });
const Template18 = _$createComponent(Pre, { get children() {
	return [
		(() => {
			var _el$17 = _$createElement("span");
			_$insertNode(_el$17, _$createTextNode("1"));
			return _el$17;
		})(),
		(() => {
			var _el$18 = _$createElement("span");
			_$insertNode(_el$18, _$createTextNode("2"));
			return _el$18;
		})(),
		(() => {
			var _el$19 = _$createElement("span");
			_$insertNode(_el$19, _$createTextNode("3"));
			return _el$19;
		})()
	];
} });
const Template19 = _$createComponent(Component, _$mergeProps(() => {
	return s.dynamic();
}));
const Template20 = _$createComponent(Component, { class: prop.red ? "red" : "green" });
const template21 = _$createComponent(Component, { get [key()]() {
	return props.value;
} });
const template22 = _$createComponent(Component, { passObject: { ...a } });
const template23 = _$createComponent(Component, { ref: binding });
const template24 = _$createComponent(Component, { ref: binding.prop });
const template25 = _$createComponent(Component, { ref: refFn });
const template26 = _$createComponent(Component, { ref: refConst });
const template27 = _$createComponent(Component, { ref: refUnknown });
const template28 = _$createComponent(Component, { ref: binding?.prop });
const template29 = _$createComponent(Component, { ref: binding?.[prop] });
const template30 = _$createComponent(Component, { ref: binding.nested?.prop });
