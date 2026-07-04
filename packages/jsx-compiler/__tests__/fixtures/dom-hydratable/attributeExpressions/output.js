import { template as _$template } from "r-dom";
import { getNextElement as _$getNextElement } from "r-dom";
import { insert as _$insert } from "r-dom";
import { scope as _$scope } from "r-dom";
import { spread as _$spread } from "r-dom";
import { mergeProps as _$mergeProps } from "r-dom";
import { ref as _$ref } from "r-dom";
import { style as _$style } from "r-dom";
import { setStyleProperty as _$setStyleProperty } from "r-dom";
import { className as _$className } from "r-dom";
import { effect as _$effect } from "r-dom";
import { setAttribute as _$setAttribute } from "r-dom";
import { addEvent as _$addEvent } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<div><h1><a href=/>Welcome`);
var _tmpl$2 = /* @__PURE__ */ _$template(`<div><div></div><div></div><div innerHTML="&lt;div/>">`);
var _tmpl$3 = /* @__PURE__ */ _$template(`<div foo>`);
var _tmpl$4 = /* @__PURE__ */ _$template(`<div>`);
var _tmpl$5 = /* @__PURE__ */ _$template(`<div class=a className=b>`);
var _tmpl$6 = /* @__PURE__ */ _$template(`<div onclick="console.log('hi')">`);
var _tmpl$7 = /* @__PURE__ */ _$template(`<input type=checkbox checked=true>`);
var _tmpl$8 = /* @__PURE__ */ _$template(`<input type=checkbox>`);
var _tmpl$9 = /* @__PURE__ */ _$template(`<div class="\`a">\`$\``);
var _tmpl$10 = /* @__PURE__ */ _$template(`<button class="static hi" type=button>Write`);
var _tmpl$11 = /* @__PURE__ */ _$template(`<button>Hi`);
var _tmpl$12 = /* @__PURE__ */ _$template(`<div class="bg-red-500 flex flex-col">`);
var _tmpl$13 = /* @__PURE__ */ _$template(`<div><input readonly=""><input>`);
var _tmpl$14 = /* @__PURE__ */ _$template(`<div data="\\"hi\\"" data2="\\"">`);
var _tmpl$15 = /* @__PURE__ */ _$template(`<a>`);
var _tmpl$16 = /* @__PURE__ */ _$template(`<div><a>`);
var _tmpl$17 = /* @__PURE__ */ _$template(`<div>Hi`);
var _tmpl$18 = /* @__PURE__ */ _$template(`<label><span>Input is </span><input><div>`);
var _tmpl$19 = /* @__PURE__ */ _$template(`<div class="class1 class2 class3 class4 class5 class6" style="color:red;background-color:blue !important;border:1px solid black;font-size:12px;" random="random1 random2\\n    random3 random4">`);
var _tmpl$20 = /* @__PURE__ */ _$template(`<button>`);
var _tmpl$21 = /* @__PURE__ */ _$template(`<input value=10>`);
var _tmpl$22 = /* @__PURE__ */ _$template(`<select><option>Red</option><option>Blue`);
var _tmpl$23 = /* @__PURE__ */ _$template(`<img src="">`);
var _tmpl$24 = /* @__PURE__ */ _$template(`<div><img src="">`);
var _tmpl$25 = /* @__PURE__ */ _$template(`<img src="" loading=lazy>`);
var _tmpl$26 = /* @__PURE__ */ _$template(`<div><img src="" loading=lazy>`);
var _tmpl$27 = /* @__PURE__ */ _$template(`<iframe src="">`);
var _tmpl$28 = /* @__PURE__ */ _$template(`<div><iframe src="">`);
var _tmpl$29 = /* @__PURE__ */ _$template(`<iframe src="" loading=lazy>`);
var _tmpl$30 = /* @__PURE__ */ _$template(`<div><iframe src="" loading=lazy>`);
var _tmpl$31 = /* @__PURE__ */ _$template(`<div title="<u>data</u>">`);
var _tmpl$32 = /* @__PURE__ */ _$template(`<div true=true truestr=true truestrjs=true>`);
var _tmpl$33 = /* @__PURE__ */ _$template(`<div false=false falsestr=false falsestrjs=false>`);
var _tmpl$34 = /* @__PURE__ */ _$template(`<div true=true false=false>`);
var _tmpl$35 = /* @__PURE__ */ _$template(`<div a b="" c="" d=true e=false f=0 g="" h="" j=null l>`);
var _tmpl$36 = /* @__PURE__ */ _$template(`<math display=block><mrow>`);
var _tmpl$37 = /* @__PURE__ */ _$template(`<mrow><mi>x</mi><mo>=`);
var _tmpl$38 = /* @__PURE__ */ _$template(`<div style=background:red>`);
var _tmpl$39 = /* @__PURE__ */ _$template(`<div style=background:red;color:green;margin:3;padding:0.4>`);
var _tmpl$40 = /* @__PURE__ */ _$template(`<div style=background:red;color:green>`);
var _tmpl$41 = /* @__PURE__ */ _$template(`<video>`);
var _tmpl$42 = /* @__PURE__ */ _$template(`<video playsinline=true>`);
var _tmpl$43 = /* @__PURE__ */ _$template(`<video playsinline=false>`);
var _tmpl$44 = /* @__PURE__ */ _$template(`<video poster=1.jpg>`);
var _tmpl$45 = /* @__PURE__ */ _$template(`<div><video poster=1.jpg>`);
var _tmpl$46 = /* @__PURE__ */ _$template(`<div><video>`);
var _tmpl$47 = /* @__PURE__ */ _$template(`<button type=button>`);
var _tmpl$48 = /* @__PURE__ */ _$template(`<div style=duplicate1 style=duplicate2>`);
var _tmpl$49 = /* @__PURE__ */ _$template(`<div class=todo>`);
var _tmpl$50 = /* @__PURE__ */ _$template(`<div class="todo item">`);
import * as styles from "./styles.module.css";
import { binding } from "somewhere";
function refFn() {}
const refConst = null;
const selected = true;
let id = "my-h1";
let link;
const template = (() => {
	var _el$ = _$getNextElement(_tmpl$);
	_$spread(_el$, _$mergeProps({ id: "main" }, results, {
		get class() {
			return { selected: unknown };
		},
		get style() {
			return { color };
		}
	}), true);
	var _el$2 = _el$.firstChild;
	_$spread(_el$2, _$mergeProps({ id }, () => {
		return results();
	}, {
		foo: true,
		disabled: true,
		get title() {
			return welcoming();
		},
		get style() {
			return {
				"background-color": color(),
				"margin-right": "40px"
			};
		},
		get class() {
			return ["base", {
				dynamic: dynamic(),
				selected
			}];
		}
	}), true);
	var _el$3 = _el$2.firstChild;
	{
		var _ref$ = link;
		typeof _ref$ === "function" || Array.isArray(_ref$) ? _$ref(() => {
			return _ref$;
		}, _el$3) : link = _el$3;
	}
	{
		_el$3.classList.toggle("ccc ddd", !!true);
	}
	return _el$;
})();
const template2 = (() => {
	var _el$4 = _$getNextElement(_tmpl$2);
	_$spread(_el$4, () => {
		return getProps("test");
	}, true);
	var _el$5 = _el$4.firstChild;
	_$effect(() => {
		return rowId;
	}, (_v$) => {
		_el$5.textContent = _v$;
	});
	var _el$6 = _el$4.firstChild.nextSibling;
	_$effect(() => {
		return row.label;
	}, (_v$) => {
		_el$6.textContent = _v$;
	});
	return _el$4;
})();
const template3 = (() => {
	var _el$7 = _$getNextElement(_tmpl$3);
	_$effect(() => {
		return state.id;
	}, (_v$) => {
		_$setAttribute(_el$7, "id", _v$);
	});
	_$effect(() => {
		return state.color;
	}, (_v$) => {
		_$setStyleProperty(_el$7, "background-color", _v$);
	});
	_$effect(() => {
		return state.name;
	}, (_v$) => {
		_$setAttribute(_el$7, "name", _v$);
	});
	_$effect(() => {
		return state.content;
	}, (_v$) => {
		_el$7.textContent = _v$;
	});
	return _el$7;
})();
var _el$8 = _$getNextElement(_tmpl$4);
_$effect(() => {
	return state.class;
}, (_v$, _$p) => {
	_$className(_el$8, _v$, _$p);
});
{
	_el$8.classList.toggle("ccc:ddd", !!true);
}
const template4 = _el$8;
const template5 = _$getNextElement(_tmpl$5);
var _el$10 = _$getNextElement(_tmpl$4);
_$effect(() => {
	return someStyle();
}, (_v$, _$p) => {
	_$style(_el$10, _v$, _$p);
});
_el$10.textContent = "Hi";
const template6 = _el$10;
let undefVar;
const template7 = (() => {
	var _el$11 = _$getNextElement(_tmpl$4);
	_$effect(() => {
		return {
			"background-color": color(),
			"margin-right": "40px",
			...props.style
		};
	}, (_v$, _$p) => {
		_$style(_el$11, _v$, _$p);
	});
	{
		_el$11.classList.toggle("other-class2", !!undefVar);
	}
	return _el$11;
})();
let refTarget;
var _el$12 = _$getNextElement(_tmpl$4);
{
	var _ref$2 = refTarget;
	typeof _ref$2 === "function" || Array.isArray(_ref$2) ? _$ref(() => {
		return _ref$2;
	}, _el$12) : refTarget = _el$12;
}
const template8 = _el$12;
var _el$13 = _$getNextElement(_tmpl$4);
_$ref(() => {
	return (e) => console.log(e);
}, _el$13);
const template9 = _el$13;
var _el$14 = _$getNextElement(_tmpl$4);
{
	var _ref$3 = refFactory();
	(typeof _ref$3 === "function" || Array.isArray(_ref$3)) && _$ref(() => {
		return _ref$3;
	}, _el$14);
}
const template10 = _el$14;
var _el$15 = _$getNextElement(_tmpl$6);
_el$15.htmlFor = thing;
_el$15.number = 123;
const template12 = _el$15;
const template13 = _$getNextElement(_tmpl$7);
var _el$17 = _$getNextElement(_tmpl$8);
_$effect(() => {
	return state.visible;
}, (_v$) => {
	_el$17.checked = _v$;
});
const template14 = _el$17;
const template15 = _$getNextElement(_tmpl$9);
const template16 = _$getNextElement(_tmpl$10);
const template17 = (() => {
	var _el$20 = _$getNextElement(_tmpl$11);
	{
		_el$20.classList.toggle("a", !!true);
		_el$20.classList.toggle("b", !!true);
		_el$20.classList.toggle("c", !!true);
	}
	_$addEvent(_el$20, "click", increment, true);
	return _el$20;
})();
const template18 = (() => {
	var _el$21 = _$getNextElement(_tmpl$4);
	_$spread(_el$21, { get [key()]() {
		return props.value;
	} }, false);
	return _el$21;
})();
const template19 = _$getNextElement(_tmpl$12);
const template20 = (() => {
	var _el$23 = _$getNextElement(_tmpl$13);
	var _el$24 = _el$23.firstChild;
	_$effect(() => {
		return s();
	}, (_v$) => {
		_el$24.value = _v$;
	});
	_$effect(() => {
		return min();
	}, (_v$) => {
		_$setAttribute(_el$24, "min", _v$);
	});
	_$effect(() => {
		return max();
	}, (_v$) => {
		_$setAttribute(_el$24, "max", _v$);
	});
	_$addEvent(_el$24, "input", doSomething, true);
	var _el$25 = _el$23.firstChild.nextSibling;
	_$effect(() => {
		return s2();
	}, (_v$) => {
		_el$25.checked = _v$;
	});
	_$effect(() => {
		return min();
	}, (_v$) => {
		_$setAttribute(_el$25, "min", _v$);
	});
	_$effect(() => {
		return max();
	}, (_v$) => {
		_$setAttribute(_el$25, "max", _v$);
	});
	_$addEvent(_el$25, "input", doSomethingElse, true);
	_$effect(() => {
		return value;
	}, (_v$) => {
		_$setAttribute(_el$25, "readonly", _v$);
	});
	return _el$23;
})();
var _el$26 = _$getNextElement(_tmpl$4);
_$effect(() => {
	return {
		c: "static",
		...rest
	};
}, (_v$, _$p) => {
	_$style(_el$26, _v$, _$p);
});
const template21 = _el$26;
const template22 = _$getNextElement(_tmpl$14);
var _el$28 = _$getNextElement(_tmpl$4);
_$effect(() => {
	return "t" in test;
}, (_v$) => {
	_$setAttribute(_el$28, "disabled", _v$);
});
_$insert(_el$28, "t" in test && "true");
const template23 = _el$28;
var _el$29 = _$getNextElement(_tmpl$15);
_$spread(_el$29, _$mergeProps(props, { something: true }), false);
const template24 = _el$29;
const template25 = (() => {
	var _el$30 = _$getNextElement(_tmpl$16);
	_$insert(_el$30, _$scope(() => {
		return props.children;
	}), _el$30.firstChild);
	var _el$31 = _el$30.firstChild;
	_$spread(_el$31, _$mergeProps(props, { something: true }), false);
	return _el$30;
})();
const template26 = (() => {
	var _el$32 = _$getNextElement(_tmpl$17);
	_$spread(_el$32, _$mergeProps({
		start: "Hi",
		middle
	}, spread), true);
	return _el$32;
})();
const template27 = (() => {
	var _el$33 = _$getNextElement(_tmpl$17);
	_$spread(_el$33, _$mergeProps({ start: "Hi" }, first, { middle }, second), true);
	return _el$33;
})();
const template28 = (() => {
	var _el$34 = _$getNextElement(_tmpl$18);
	_$spread(_el$34, () => {
		return api();
	}, true);
	var _el$35 = _el$34.firstChild;
	_$spread(_el$35, () => {
		return api();
	}, true);
	_$insert(_el$35, () => {
		return api() ? "checked" : "unchecked";
	});
	var _el$36 = _el$34.firstChild.nextSibling;
	_$spread(_el$36, () => {
		return api();
	}, false);
	var _el$37 = _el$34.firstChild.nextSibling.nextSibling;
	_$spread(_el$37, () => {
		return api();
	}, false);
	return _el$34;
})();
var _el$38 = _$getNextElement(_tmpl$4);
_$effect(() => {
	return !!someValue;
}, (_v$) => {
	_$setAttribute(_el$38, "attribute", _v$);
});
_$insert(_el$38, !!someValue);
const template29 = _el$38;
const template30 = _$getNextElement(_tmpl$19);
var _el$40 = _$getNextElement(_tmpl$4);
_$effect(() => {
	return getStore.itemProperties.color;
}, (_v$) => {
	_$setStyleProperty(_el$40, "background-color", _v$);
});
const template31 = _el$40;
const template32 = _$getNextElement(_tmpl$4);
const template33 = [
	(() => {
		var _el$42 = _$getNextElement(_tmpl$20);
		_$effect(() => {
			return styles.button;
		}, (_v$, _$p) => {
			_$className(_el$42, _v$, _$p);
		});
		return _el$42;
	})(),
	(() => {
		var _el$43 = _$getNextElement(_tmpl$20);
		_$effect(() => {
			return styles["foo--bar"];
		}, (_v$, _$p) => {
			_$className(_el$43, _v$, _$p);
		});
		return _el$43;
	})(),
	(() => {
		var _el$44 = _$getNextElement(_tmpl$20);
		_$effect(() => {
			return styles.foo.bar;
		}, (_v$, _$p) => {
			_$className(_el$44, _v$, _$p);
		});
		return _el$44;
	})(),
	(() => {
		var _el$45 = _$getNextElement(_tmpl$20);
		_$effect(() => {
			return styles[foo()];
		}, (_v$, _$p) => {
			_$className(_el$45, _v$, _$p);
		});
		return _el$45;
	})()
];
var _el$46 = _$getNextElement(_tmpl$4);
{
	var _ref$4 = a().b.c;
	typeof _ref$4 === "function" || Array.isArray(_ref$4) ? _$ref(() => {
		return _ref$4;
	}, _el$46) : a().b.c = _el$46;
}
const template35 = _el$46;
var _el$47 = _$getNextElement(_tmpl$4);
{
	var _ref$5 = a().b?.c;
	(typeof _ref$5 === "function" || Array.isArray(_ref$5)) && _$ref(() => {
		return _ref$5;
	}, _el$47);
}
const template36 = _el$47;
var _el$48 = _$getNextElement(_tmpl$4);
{
	var _ref$6 = a() ? b : c;
	(typeof _ref$6 === "function" || Array.isArray(_ref$6)) && _$ref(() => {
		return _ref$6;
	}, _el$48);
}
const template37 = _el$48;
var _el$49 = _$getNextElement(_tmpl$4);
{
	var _ref$7 = a() ?? b;
	(typeof _ref$7 === "function" || Array.isArray(_ref$7)) && _$ref(() => {
		return _ref$7;
	}, _el$49);
}
const template38 = _el$49;
const template39 = _$getNextElement(_tmpl$21);
var _el$51 = _$getNextElement(_tmpl$4);
_$effect(() => {
	return a();
}, (_v$) => {
	_$setStyleProperty(_el$51, "color", _v$);
});
const template40 = _el$51;
const template41 = (() => {
	var _el$52 = _$getNextElement(_tmpl$22);
	_$effect(() => {
		return state.color;
	}, (_v$) => {
		_el$52.value = _v$;
	});
	var _el$53 = _el$52.firstChild;
	_$effect(() => {
		return Color.Red;
	}, (_v$) => {
		_el$53.value = _v$;
	});
	var _el$54 = _el$52.firstChild.nextSibling;
	_$effect(() => {
		return Color.Blue;
	}, (_v$) => {
		_el$54.value = _v$;
	});
	return _el$52;
})();
const template42 = _$getNextElement(_tmpl$23);
const template43 = _$getNextElement(_tmpl$24);
const template44 = _$getNextElement(_tmpl$25);
const template45 = _$getNextElement(_tmpl$26);
const template46 = _$getNextElement(_tmpl$27);
const template47 = _$getNextElement(_tmpl$28);
const template48 = _$getNextElement(_tmpl$29);
const template49 = _$getNextElement(_tmpl$30);
const template50 = _$getNextElement(_tmpl$31);
var _el$64 = _$getNextElement(_tmpl$4);
{
	var _ref$8 = binding;
	typeof _ref$8 === "function" || Array.isArray(_ref$8) ? _$ref(() => {
		return _ref$8;
	}, _el$64) : binding = _el$64;
}
const template51 = _el$64;
var _el$65 = _$getNextElement(_tmpl$4);
{
	var _ref$9 = binding.prop;
	typeof _ref$9 === "function" || Array.isArray(_ref$9) ? _$ref(() => {
		return _ref$9;
	}, _el$65) : binding.prop = _el$65;
}
const template52 = _el$65;
var _el$66 = _$getNextElement(_tmpl$4);
{
	var _ref$10 = refFn;
	typeof _ref$10 === "function" || Array.isArray(_ref$10) ? _$ref(() => {
		return _ref$10;
	}, _el$66) : refFn = _el$66;
}
const template53 = _el$66;
var _el$67 = _$getNextElement(_tmpl$4);
{
	var _ref$11 = refConst;
	typeof _ref$11 === "function" || Array.isArray(_ref$11) ? _$ref(() => {
		return _ref$11;
	}, _el$67) : refConst = _el$67;
}
const template54 = _el$67;
var _el$68 = _$getNextElement(_tmpl$4);
{
	var _ref$12 = refUnknown;
	typeof _ref$12 === "function" || Array.isArray(_ref$12) ? _$ref(() => {
		return _ref$12;
	}, _el$68) : refUnknown = _el$68;
}
const template55 = _el$68;
const template56 = _$getNextElement(_tmpl$32);
const template57 = _$getNextElement(_tmpl$33);
var _el$71 = _$getNextElement(_tmpl$4);
_el$71.true = true;
_el$71.false = false;
const template58 = _el$71;
const template59 = _$getNextElement(_tmpl$34);
var _el$73 = _$getNextElement(_tmpl$35);
_$effect(() => {
	return undefined;
}, (_v$) => {
	_$setAttribute(_el$73, "i", _v$);
});
_$effect(() => {
	return void 0;
}, (_v$) => {
	_$setAttribute(_el$73, "k", _v$);
});
const template60 = _el$73;
const template61 = _$getNextElement(_tmpl$36);
const template62 = _$getNextElement(_tmpl$37);
const template63 = _$getNextElement(_tmpl$38);
const template64 = _$getNextElement(_tmpl$39);
const template65 = _$getNextElement(_tmpl$40);
var _el$79 = _$getNextElement(_tmpl$40);
_$effect(() => {
	return signal();
}, (_v$) => {
	_$setStyleProperty(_el$79, "border", _v$);
});
const template66 = _el$79;
var _el$80 = _$getNextElement(_tmpl$40);
_$setStyleProperty(_el$80, "border", somevalue);
const template67 = _el$80;
var _el$81 = _$getNextElement(_tmpl$40);
_$effect(() => {
	return some.access;
}, (_v$) => {
	_$setStyleProperty(_el$81, "border", _v$);
});
const template68 = _el$81;
const template69 = _$getNextElement(_tmpl$40);
var _el$83 = _$getNextElement(_tmpl$41);
_$effect(() => {
	return value;
}, (_v$) => {
	_$setAttribute(_el$83, "playsinline", _v$);
});
const template70 = _el$83;
const template71 = _$getNextElement(_tmpl$42);
const template72 = _$getNextElement(_tmpl$43);
const template73 = _$getNextElement(_tmpl$44);
const template74 = _$getNextElement(_tmpl$45);
var _el$88 = _$getNextElement(_tmpl$41);
_el$88.poster = "1.jpg";
const template75 = _el$88;
var _el$89 = _$getNextElement(_tmpl$46);
var _el$90 = _el$89.firstChild;
_el$90.poster = "1.jpg";
const template76 = _el$89;
var _el$91 = _$getNextElement(_tmpl$4);
_$effect(() => {
	return props.width;
}, (_v$) => {
	_$setStyleProperty(_el$91, "width", _v$);
});
_$effect(() => {
	return props.height;
}, (_v$) => {
	_$setStyleProperty(_el$91, "height", _v$);
});
// STATIC TESTS
const template77 = _el$91;
const template78 = (() => {
	var _el$92 = _$getNextElement(_tmpl$4);
	_$effect(() => {
		return props.width;
	}, (_v$) => {
		_$setStyleProperty(_el$92, "width", _v$);
	});
	_$effect(() => {
		return props.height;
	}, (_v$) => {
		_$setStyleProperty(_el$92, "height", _v$);
	});
	_$effect(() => {
		return color();
	}, (_v$) => {
		_$setAttribute(_el$92, "something", _v$);
	});
	return _el$92;
})();
const template79 = (() => {
	var _el$93 = _$getNextElement(_tmpl$4);
	_$effect(() => {
		return props.width;
	}, (_v$) => {
		_$setStyleProperty(_el$93, "width", _v$);
	});
	_$effect(() => {
		return props.height;
	}, (_v$) => {
		_$setStyleProperty(_el$93, "height", _v$);
	});
	_$effect(() => {
		return color();
	}, (_v$) => {
		_$setAttribute(_el$93, "something", _v$);
	});
	return _el$93;
})();
// STATIC TESTS SPREADS
const propsSpread = {
	something: color(),
	style: {
		"background-color": color(),
		color: color(),
		"margin-right": props.right
	}
};
var _el$94 = _$getNextElement(_tmpl$4);
_$spread(_el$94, propsSpread, false);
const template80 = _el$94;
var _el$95 = _$getNextElement(_tmpl$4);
_$spread(_el$95, propsSpread, false);
const template81 = _el$95;
const template82 = (() => {
	var _el$96 = _$getNextElement(_tmpl$4);
	_$spread(_el$96, _$mergeProps(propsSpread, {
		get "data-dynamic"() {
			return color();
		},
		"data-static": color()
	}), false);
	return _el$96;
})();
const template83 = (() => {
	var _el$97 = _$getNextElement(_tmpl$4);
	_$spread(_el$97, _$mergeProps(propsSpread, {
		get "data-dynamic"() {
			return color();
		},
		"data-static": color()
	}), false);
	return _el$97;
})();
const template84 = (() => {
	var _el$98 = _$getNextElement(_tmpl$4);
	_$spread(_el$98, _$mergeProps(propsSpread1, propsSpread2, propsSpread3, {
		get "data-dynamic"() {
			return color();
		},
		"data-static": color()
	}), false);
	return _el$98;
})();
// STATIC PROPERTY OF OBJECT ACCESS
// https://github.com/ryansolid/dom-expressions/issues/252#issuecomment-1572220563
const styleProp = { style: {
	width: props.width,
	height: props.height
} };
var _el$99 = _$getNextElement(_tmpl$4);
_$effect(() => {
	return styleProp.style;
}, (_v$, _$p) => {
	_$style(_el$99, _v$, _$p);
});
const template85 = _el$99;
var _el$100 = _$getNextElement(_tmpl$4);
_$effect(() => {
	return styleProp.style;
}, (_v$, _$p) => {
	_$style(_el$100, _v$, _$p);
});
const template86 = _el$100;
const style = {
	background: "red",
	border: "solid black " + count() + "px"
};
const template87 = (() => {
	var _el$101 = _$getNextElement(_tmpl$47);
	_$effect(() => {
		return count();
	}, (_v$) => {
		_$setAttribute(_el$101, "aria-label", _v$);
	});
	_$effect(() => {
		return style;
	}, (_v$, _$p) => {
		_$style(_el$101, _v$, _$p);
	});
	_$effect(() => {
		return style;
	}, (_v$, _$p) => {
		_$className(_el$101, _v$, _$p);
	});
	_$insert(_el$101, _$scope(() => {
		return count();
	}));
	return _el$101;
})();
const template88 = (() => {
	var _el$102 = _$getNextElement(_tmpl$47);
	_$effect(() => {
		return count();
	}, (_v$) => {
		_$setAttribute(_el$102, "aria-label", _v$);
	});
	_$effect(() => {
		return style;
	}, (_v$, _$p) => {
		_$style(_el$102, _v$, _$p);
	});
	_$effect(() => {
		return style;
	}, (_v$, _$p) => {
		_$className(_el$102, _v$, _$p);
	});
	_$insert(_el$102, _$scope(() => {
		return count();
	}));
	return _el$102;
})();
const template89 = _$getNextElement(_tmpl$48);
var _el$104 = _$getNextElement(_tmpl$49);
_$effect(() => {
	return !!isActive();
}, (_v$) => {
	_el$104.classList.toggle("active", _v$);
});
const template90 = _el$104;
var _el$105 = _$getNextElement(_tmpl$4);
_$effect(() => {
	return ["todo", props.active];
}, (_v$, _$p) => {
	_$className(_el$105, _v$, _$p);
});
const template91 = _el$105;
var _el$106 = _$getNextElement(_tmpl$50);
_$effect(() => {
	return !!isActive();
}, (_v$) => {
	_el$106.classList.toggle("active", _v$);
});
const template92 = _el$106;
var _el$107 = _$getNextElement(_tmpl$4);
_$effect(() => {
	return ["todo", {
		active: isActive(),
		[props.name]: props.enabled
	}];
}, (_v$, _$p) => {
	_$className(_el$107, _v$, _$p);
});
const template93 = _el$107;
var _el$108 = _$getNextElement(_tmpl$4);
_$effect(() => {
	return [
		"todo",
		{ active: isActive() },
		props.extra
	];
}, (_v$, _$p) => {
	_$className(_el$108, _v$, _$p);
});
const template94 = _el$108;
var _el$109 = _$getNextElement(_tmpl$50);
_$effect(() => {
	return !!isActive();
}, (_v$) => {
	_el$109.classList.toggle("active", _v$);
});
const template95 = _el$109;
