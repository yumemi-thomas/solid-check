import { escape as _$escape } from "r-server";
import { ssr as _$ssr } from "r-server";
import { ssrElement as _$ssrElement } from "r-server";
import { mergeProps as _$mergeProps } from "r-server";
import { Show, binding } from "somewhere";
function refFn() {}
const refConst = null;
const Child = (props) => {
	const [s, set] = createSignal();
	return [_$ssr([
		"<div ref=\"",
		"\">Hello ",
		"</div>"
	], _$escape(props.ref, true), _$escape(props.name)), _$ssr([
		"<div ref=\"",
		"\">",
		"</div>"
	], _$escape(set, true), _$escape(props.children))];
};
const template = (props) => {
	let childRef;
	const { content } = props;
	return _$ssr([
		"<div>",
		"",
		"",
		"</div>"
	], Child(_$mergeProps({ name: "John" }, props, {
		ref: childRef,
		booleanProperty: true,
		get children() {
			return _$ssr("<div>From Parent</div>");
		}
	})), Child(_$mergeProps({ name: "Jason" }, () => {
		return dynamicSpread();
	}, {
		ref: props.ref,
		get children() {
			return _$ssr(["<div>", "</div>"], _$escape(content));
		}
	})), Context.Consumer({
		ref: props.consumerRef(),
		get children() {
			return (context) => context;
		}
	}));
};
const template2 = Child({
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
const template3 = Child({ get children() {
	return [
		_$ssr("<div></div>"),
		_$ssr("<div></div>"),
		_$ssr("<div></div>"),
		"After"
	];
} });
const [s, set] = createSignal();
const template4 = Child({
	ref: set,
	get children() {
		return _$ssr("<div></div>");
	}
});
const template5 = Child({
	get dynamic() {
		return state.dynamic;
	},
	get children() {
		return state.dynamic;
	}
});
// builtIns
const template6 = For({
	get each() {
		return state.list;
	},
	get fallback() {
		return Loading({});
	},
	get children() {
		return (item) => Show({
			get when() {
				return state.condition;
			},
			get children() {
				return item;
			}
		});
	}
});
const template7 = Child({ get children() {
	return [_$ssr("<div></div>"), state.dynamic];
} });
const template8 = Child({ get children() {
	return [(item) => item, (item) => item];
} });
const template9 = _garbage({ children: "Hi" });
const template10 = _$ssr([
	"<div>",
	" | ",
	" | ",
	" | ",
	" | ",
	" | ",
	"</div>"
], Link({ children: "new" }), Link({ children: "comments" }), Link({ children: "show" }), Link({ children: "ask" }), Link({ children: "jobs" }), Link({ children: "submit" }));
const template11 = _$ssr([
	"<div>",
	" | ",
	"",
	" | ",
	"",
	" | ",
	"</div>"
], Link({ children: "new" }), Link({ children: "comments" }), Link({ children: "show" }), Link({ children: "ask" }), Link({ children: "jobs" }), Link({ children: "submit" }));
const template12 = _$ssr([
	"<div> | ",
	" |  |  | ",
	" | </div>"
], Link({ children: "comments" }), Link({ children: "show" }));
class Template13 {
	render() {
		const _self$ = this;
		Component({
			get prop() {
				return _self$.something;
			},
			onClick: () => _self$.shouldStay,
			get children() {
				return Nested({
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
const Template14 = Component({ get children() {
	return data();
} });
const Template15 = Component(props);
const Template16 = Component(_$mergeProps({ something }, props));
const Template17 = Pre({ get children() {
	return [
		_$ssr("<span>1</span>"),
		" ",
		_$ssr("<span>2</span>"),
		" ",
		_$ssr("<span>3</span>")
	];
} });
const Template18 = Pre({ get children() {
	return [
		_$ssr("<span>1</span>"),
		_$ssr("<span>2</span>"),
		_$ssr("<span>3</span>")
	];
} });
const Template19 = Component(_$mergeProps(() => {
	return s.dynamic();
}));
const Template20 = Component({ class: prop.red ? "red" : "green" });
const template21 = Component({ get [key()]() {
	return props.value;
} });
const template22 = Component({ passObject: { ...a } });
const template23 = Component({
	disabled: "t" in test,
	get children() {
		return "t" in test && "true";
	}
});
const template24 = Component({ get children() {
	return state.dynamic;
} });
const template25 = Component({ get children() {
	return _$ssr("<div></div>");
} });
const template26 = [Component({ when: (() => {
	const foo = test();
	if ("t" in foo) {
		return foo;
	}
})() }), Component({ when: ((val = 123) => {
	return val * 2;
})() })];
const template27 = Component({ when: (() => prop.red ? "red" : "green")() });
class Template28 {
	render() {
		return Component({ when: (() => {
			const _self$2 = this;
			const foo = _self$2.value;
			if ("key" in foo) {
				return foo;
			}
		})() });
	}
}
class Template29 extends ParentComponent {
	constructor() {
		super();
		const _self$3 = this;
		_self$3.component({ get method() {
			return _self$3.method;
		} });
	}
	get get() {
		const _self$4 = this;
		_self$4.component({ get method() {
			return _self$4.method;
		} });
	}
	set set(v) {
		const _self$5 = this;
		_self$5.component({ get method() {
			return _self$5.method;
		} });
	}
	method() {
		const _self$6 = this;
		_self$6.component({ get method() {
			return _self$6.method;
		} });
	}
	field = (() => {
		const _self$7 = this;
		return _self$7.component({
			get method() {
				return _self$7.method;
			},
			get comp() {
				return _self$7.another({});
			}
		});
	})();
	fieldArrow = () => {
		const _self$8 = this;
		return _self$8.component({ get method() {
			return _self$8.method;
		} });
	};
	fieldFunction = function() {
		const _self$9 = this;
		_self$9.component({ get method() {
			return _self$9.method;
		} });
	};
}
const template30 = Comp({ ref: binding });
const template31 = Comp({ ref: binding.prop });
const template32 = Comp({ ref: refFn });
const template33 = Comp({ ref: refConst });
const template34 = Comp({ ref: refUnknown });
function MyComponent(props) {
	let el;
	const others = omit(props, "children");
	return _$ssrElement("div", _$mergeProps({ ref: el }, others), props.children, false);
}
