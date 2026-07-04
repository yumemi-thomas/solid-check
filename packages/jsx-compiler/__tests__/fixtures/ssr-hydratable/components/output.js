import { scope as _$scope } from "r-server";
import { escape as _$escape } from "r-server";
import { ssr as _$ssr } from "r-server";
import { ssrHydrationKey as _$ssrHydrationKey } from "r-server";
import { ssrElement as _$ssrElement } from "r-server";
import { mergeProps as _$mergeProps } from "r-server";
import { For as _$For } from "r-server";
import { Show as _$Show } from "r-server";
import { Show } from "somewhere";
const Child = (props) => {
	const [s, set] = createSignal();
	return [_$ssr([
		"<div",
		" ref=\"",
		"\">Hello ",
		"</div>"
	], _$ssrHydrationKey(), _$escape(props.ref, true), _$escape(props.name)), _$ssr([
		"<div",
		" ref=\"",
		"\">",
		"</div>"
	], _$ssrHydrationKey(), _$escape(set, true), _$scope(() => {
		return _$escape(props.children);
	}))];
};
const template = (props) => {
	let childRef;
	const { content } = props;
	return _$ssr([
		"<div",
		">",
		"",
		"",
		"</div>"
	], _$ssrHydrationKey(), Child(_$mergeProps({ name: "John" }, props, {
		ref: childRef,
		booleanProperty: true,
		get children() {
			return _$ssr(["<div", ">From Parent</div>"], _$ssrHydrationKey());
		}
	})), Child(_$mergeProps({ name: "Jason" }, () => {
		return dynamicSpread();
	}, {
		ref: props.ref,
		get children() {
			return _$ssr([
				"<div",
				">",
				"</div>"
			], _$ssrHydrationKey(), _$escape(content));
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
		_$ssr(["<div", "></div>"], _$ssrHydrationKey()),
		_$ssr(["<div", "></div>"], _$ssrHydrationKey()),
		_$ssr(["<div", "></div>"], _$ssrHydrationKey()),
		"After"
	];
} });
const [s, set] = createSignal();
const template4 = Child({
	ref: set,
	get children() {
		return _$ssr(["<div", "></div>"], _$ssrHydrationKey());
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
const template6 = _$For({
	get each() {
		return state.list;
	},
	get fallback() {
		return Loading({});
	},
	get children() {
		return (item) => _$Show({
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
	return [_$ssr(["<div", "></div>"], _$ssrHydrationKey()), state.dynamic];
} });
const template8 = Child({ get children() {
	return [(item) => item, (item) => item];
} });
const template9 = _garbage({ children: "Hi" });
const template10 = _$ssr([
	"<div",
	">",
	" | ",
	" | ",
	" | ",
	" | ",
	" | ",
	"</div>"
], _$ssrHydrationKey(), Link({ children: "new" }), Link({ children: "comments" }), Link({ children: "show" }), Link({ children: "ask" }), Link({ children: "jobs" }), Link({ children: "submit" }));
const template11 = _$ssr([
	"<div",
	">",
	" | ",
	"",
	" | ",
	"",
	" | ",
	"</div>"
], _$ssrHydrationKey(), Link({ children: "new" }), Link({ children: "comments" }), Link({ children: "show" }), Link({ children: "ask" }), Link({ children: "jobs" }), Link({ children: "submit" }));
const template12 = _$ssr([
	"<div",
	"> | ",
	" |  |  | ",
	" | </div>"
], _$ssrHydrationKey(), Link({ children: "comments" }), Link({ children: "show" }));
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
		_$ssr(["<span", ">1</span>"], _$ssrHydrationKey()),
		" ",
		_$ssr(["<span", ">2</span>"], _$ssrHydrationKey()),
		" ",
		_$ssr(["<span", ">3</span>"], _$ssrHydrationKey())
	];
} });
const Template18 = Pre({ get children() {
	return [
		_$ssr(["<span", ">1</span>"], _$ssrHydrationKey()),
		_$ssr(["<span", ">2</span>"], _$ssrHydrationKey()),
		_$ssr(["<span", ">3</span>"], _$ssrHydrationKey())
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
	return _$ssr(["<div", "></div>"], _$ssrHydrationKey());
} });
function MyComponent(props) {
	let el;
	const others = omit(props, "children");
	return _$ssrElement("div", _$mergeProps({ ref: el }, others), props.children, false);
}
