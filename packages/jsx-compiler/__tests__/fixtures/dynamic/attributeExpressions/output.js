import { createTextNode as _$createTextNode2 } from "r-custom";
import { insertNode as _$insertNode2 } from "r-custom";
import { createElement as _$createElement2 } from "r-custom";
import { template as _$template } from "r-dom";
import { insert as _$insert } from "r-dom";
import { spread as _$spread } from "r-dom";
import { mergeProps as _$mergeProps } from "r-dom";
import { ref as _$ref } from "r-dom";
import { style as _$style } from "r-dom";
import { setStyleProperty as _$setStyleProperty } from "r-dom";
import { className as _$className } from "r-dom";
import { effect as _$effect } from "r-dom";
import { setAttribute as _$setAttribute } from "r-dom";
import { addEvent as _$addEvent } from "r-dom";
import { delegateEvents as _$delegateEvents } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<div><h1><a href=/>Welcome`);
var _tmpl$2 = /* @__PURE__ */ _$template(`<div><div></div><div></div><div>`);
var _tmpl$3 = /* @__PURE__ */ _$template(`<div foo>`);
var _tmpl$4 = /* @__PURE__ */ _$template(`<div>`);
var _tmpl$5 = /* @__PURE__ */ _$template(`<div class=a className=b>`);
var _tmpl$6 = /* @__PURE__ */ _$template(`<div onclick="console.log('hi')">`);
var _tmpl$7 = /* @__PURE__ */ _$template(`<input type=checkbox checked>`);
var _tmpl$8 = /* @__PURE__ */ _$template(`<input type=checkbox>`);
var _tmpl$9 = /* @__PURE__ */ _$template(`<div class="\`a">\`$\``);
var _tmpl$10 = /* @__PURE__ */ _$template(`<button class="static hi"type=button>Write`);
var _tmpl$11 = /* @__PURE__ */ _$template(`<button>Hi`);
var _tmpl$12 = /* @__PURE__ */ _$template(`<div class="bg-red-500 flex flex-col">`);
var _tmpl$13 = /* @__PURE__ */ _$template(`<div><input readonly><input>`);
var _tmpl$14 = /* @__PURE__ */ _$template(`<div data="&quot;hi&quot;"data2="&quot;">`);
var _tmpl$15 = /* @__PURE__ */ _$template(`<a>`);
var _tmpl$16 = /* @__PURE__ */ _$template(`<div><a>`);
var _tmpl$17 = /* @__PURE__ */ _$template(`<div>Hi`);
var _tmpl$18 = /* @__PURE__ */ _$template(`<label><span>Input is </span><input><div>`);
var _tmpl$19 = /* @__PURE__ */ _$template(`<div class="class1 class2 class3 class4 class5 class6"style="color:red;background-color:blue !important;border:1px solid black;font-size:12px;"random="random1 random2
    random3 random4">`);
var _tmpl$20 = /* @__PURE__ */ _$template(`<button>`);
var _tmpl$21 = /* @__PURE__ */ _$template(`<input value=10>`);
var _tmpl$22 = /* @__PURE__ */ _$template(`<select><option>Red</option><option>Blue`);
var _tmpl$23 = /* @__PURE__ */ _$template(`<img src>`);
var _tmpl$24 = /* @__PURE__ */ _$template(`<div><img src>`);
var _tmpl$25 = /* @__PURE__ */ _$template(`<img src loading=lazy>`, 1);
var _tmpl$26 = /* @__PURE__ */ _$template(`<div><img src loading=lazy>`, 1);
var _tmpl$27 = /* @__PURE__ */ _$template(`<iframe src>`);
var _tmpl$28 = /* @__PURE__ */ _$template(`<div><iframe src>`);
var _tmpl$29 = /* @__PURE__ */ _$template(`<iframe src loading=lazy>`, 1);
var _tmpl$30 = /* @__PURE__ */ _$template(`<div><iframe src loading=lazy>`, 1);
var _tmpl$31 = /* @__PURE__ */ _$template(`<div title="<u>data</u>">`);
var _tmpl$32 = /* @__PURE__ */ _$template(`<div true truestr=true truestrjs=true>`);
var _tmpl$33 = /* @__PURE__ */ _$template(`<div falsestr=false falsestrjs=false>`);
var _tmpl$34 = /* @__PURE__ */ _$template(`<div true>`);
var _tmpl$35 = /* @__PURE__ */ _$template(`<div a b c d f=0 g h l>`);
var _tmpl$36 = /* @__PURE__ */ _$template(`<div style=background:red>`);
var _tmpl$37 = /* @__PURE__ */ _$template(`<div style=background:red;color:green;margin:3;padding:0.4>`);
var _tmpl$38 = /* @__PURE__ */ _$template(`<div style=background:red;color:green>`);
var _tmpl$39 = /* @__PURE__ */ _$template(`<video>`);
var _tmpl$40 = /* @__PURE__ */ _$template(`<video playsinline>`);
var _tmpl$41 = /* @__PURE__ */ _$template(`<video poster=1.jpg>`);
var _tmpl$42 = /* @__PURE__ */ _$template(`<div><video poster=1.jpg>`);
var _tmpl$43 = /* @__PURE__ */ _$template(`<div><video>`);
var _tmpl$44 = /* @__PURE__ */ _$template(`<button type=button>`);
import * as styles from "./styles.module.css";
import { binding } from "somewhere";
function refFn() {}
const refConst = null;
const selected = true;
let id = "my-h1";
let link;
const template = (() => {
	var _el$ = _tmpl$();
	var _el$2 = _el$.firstChild;
	var _el$3 = _el$2.firstChild;
	_$spread(_el$, _$mergeProps({ id: "main" }, results, {
		get class() {
			return { selected: unknown };
		},
		get style() {
			return { color };
		}
	}), true);
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
	var _el$4 = _tmpl$2();
	var _el$5 = _el$4.firstChild;
	var _el$6 = _el$4.firstChild.nextSibling;
	var _el$7 = _el$4.firstChild.nextSibling.nextSibling;
	_$spread(_el$4, () => {
		return getProps("test");
	}, true);
	_el$5.textContent = rowId;
	_$effect(() => {
		return row.label;
	}, (_v$) => {
		_el$6.textContent = _v$;
	});
	_el$7.innerHTML = "<div/>";
	return _el$4;
})();
const template3 = (() => {
	var _el$8 = _tmpl$3();
	_$setAttribute(
		_el$8,
		"id",
		/*@static*/
		state.id
	);
	_$setStyleProperty(_el$8, "background-color", state.color);
	_$effect(() => {
		return state.name;
	}, (_v$) => {
		_$setAttribute(_el$8, "name", _v$);
	});
	_el$8.textContent = state.content;
	return _el$8;
})();
const template4 = (() => {
	var _el$9 = _tmpl$4();
	_$effect(() => {
		return state.class;
	}, (_v$, _$p) => {
		_$className(_el$9, _v$, _$p);
	});
	{
		_el$9.classList.toggle("ccc:ddd", !!true);
	}
	return _el$9;
})();
const template5 = _tmpl$5();
const template6 = (() => {
	var _el$11 = _tmpl$4();
	_$effect(() => {
		return someStyle();
	}, (_v$, _$p) => {
		_$style(_el$11, _v$, _$p);
	});
	_el$11.textContent = "Hi";
	return _el$11;
})();
let undefVar;
const template7 = (() => {
	var _el$12 = _tmpl$4();
	_$effect(() => {
		return {
			"background-color": color(),
			"margin-right": "40px",
			...props.style
		};
	}, (_v$, _$p) => {
		_$style(_el$12, _v$, _$p);
	});
	{
		_el$12.classList.toggle("other-class2", !!undefVar);
	}
	return _el$12;
})();
let refTarget;
const template8 = (() => {
	var _el$13 = _tmpl$4();
	{
		var _ref$2 = refTarget;
		typeof _ref$2 === "function" || Array.isArray(_ref$2) ? _$ref(() => {
			return _ref$2;
		}, _el$13) : refTarget = _el$13;
	}
	return _el$13;
})();
const template9 = (() => {
	var _el$14 = _tmpl$4();
	_$ref(() => {
		return (e) => console.log(e);
	}, _el$14);
	return _el$14;
})();
const template10 = (() => {
	var _el$15 = _tmpl$4();
	{
		var _ref$3 = refFactory();
		(typeof _ref$3 === "function" || Array.isArray(_ref$3)) && _$ref(() => {
			return _ref$3;
		}, _el$15);
	}
	return _el$15;
})();
const template12 = (() => {
	var _el$16 = _tmpl$6();
	_el$16.htmlFor = thing;
	_el$16.number = 123;
	return _el$16;
})();
const template13 = _tmpl$7();
const template14 = (() => {
	var _el$18 = _tmpl$8();
	_$effect(() => {
		return state.visible;
	}, (_v$) => {
		_el$18.checked = _v$;
	});
	return _el$18;
})();
const template15 = _tmpl$9();
const template16 = _tmpl$10();
const template17 = (() => {
	var _el$21 = _tmpl$11();
	{
		_el$21.classList.toggle("a", !!true);
		_el$21.classList.toggle("b", !!true);
		_el$21.classList.toggle("c", !!true);
	}
	_$addEvent(_el$21, "click", increment, true);
	return _el$21;
})();
const template18 = (() => {
	var _el$22 = _tmpl$4();
	_$spread(_el$22, { get [key()]() {
		return props.value;
	} }, false);
	return _el$22;
})();
const template19 = _tmpl$12();
const template20 = (() => {
	var _el$24 = _tmpl$13();
	var _el$25 = _el$24.firstChild;
	var _el$26 = _el$24.firstChild.nextSibling;
	_$effect(() => {
		return s();
	}, (_v$) => {
		_el$25.value = _v$;
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
	_$addEvent(_el$25, "input", doSomething, true);
	_$effect(() => {
		return s2();
	}, (_v$) => {
		_el$26.checked = _v$;
	});
	_$effect(() => {
		return min();
	}, (_v$) => {
		_$setAttribute(_el$26, "min", _v$);
	});
	_$effect(() => {
		return max();
	}, (_v$) => {
		_$setAttribute(_el$26, "max", _v$);
	});
	_$addEvent(_el$26, "input", doSomethingElse, true);
	_$setAttribute(_el$26, "readonly", value);
	return _el$24;
})();
const template21 = (() => {
	var _el$27 = _tmpl$4();
	_$effect(() => {
		return {
			a: "static",
			...rest
		};
	}, (_v$, _$p) => {
		_$style(_el$27, _v$, _$p);
	});
	return _el$27;
})();
const template22 = _tmpl$14();
const template23 = (() => {
	var _el$29 = _tmpl$4();
	_$effect(() => {
		return "t" in test;
	}, (_v$) => {
		_$setAttribute(_el$29, "disabled", _v$);
	});
	_$insert(_el$29, "t" in test && "true");
	return _el$29;
})();
const template24 = (() => {
	var _el$30 = _tmpl$15();
	_$spread(_el$30, _$mergeProps(props, { something: true }), false);
	return _el$30;
})();
const template25 = (() => {
	var _el$31 = _tmpl$16();
	var _el$32 = _el$31.firstChild;
	_$insert(_el$31, () => {
		return props.children;
	}, _el$31.firstChild);
	_$spread(_el$32, _$mergeProps(props, { something: true }), false);
	return _el$31;
})();
const template26 = (() => {
	var _el$33 = _tmpl$17();
	_$spread(_el$33, _$mergeProps({
		start: "Hi",
		middle
	}, spread), true);
	return _el$33;
})();
const template27 = (() => {
	var _el$34 = _tmpl$17();
	_$spread(_el$34, _$mergeProps({ start: "Hi" }, first, { middle }, second), true);
	return _el$34;
})();
const template28 = (() => {
	var _el$35 = _tmpl$18();
	var _el$36 = _el$35.firstChild;
	var _el$37 = _el$35.firstChild.nextSibling;
	var _el$38 = _el$35.firstChild.nextSibling.nextSibling;
	_$spread(_el$35, () => {
		return api();
	}, true);
	_$spread(_el$36, () => {
		return api();
	}, true);
	_$insert(_el$36, () => {
		return api() ? "checked" : "unchecked";
	}, null);
	_$spread(_el$37, () => {
		return api();
	}, false);
	_$spread(_el$38, () => {
		return api();
	}, false);
	return _el$35;
})();
const template29 = (() => {
	var _el$39 = _tmpl$4();
	_$setAttribute(_el$39, "attribute", !!someValue);
	_$insert(_el$39, !!someValue);
	return _el$39;
})();
const template30 = _tmpl$19();
const template31 = (() => {
	var _el$41 = _tmpl$4();
	_$effect(() => {
		return getStore.itemProperties.color;
	}, (_v$) => {
		_$setStyleProperty(_el$41, "background-color", _v$);
	});
	return _el$41;
})();
const template32 = _tmpl$4();
const template33 = [
	(() => {
		var _el$43 = _tmpl$20();
		_$effect(() => {
			return styles.button;
		}, (_v$, _$p) => {
			_$className(_el$43, _v$, _$p);
		});
		return _el$43;
	})(),
	(() => {
		var _el$44 = _tmpl$20();
		_$effect(() => {
			return styles["foo--bar"];
		}, (_v$, _$p) => {
			_$className(_el$44, _v$, _$p);
		});
		return _el$44;
	})(),
	(() => {
		var _el$45 = _tmpl$20();
		_$effect(() => {
			return styles.foo.bar;
		}, (_v$, _$p) => {
			_$className(_el$45, _v$, _$p);
		});
		return _el$45;
	})(),
	(() => {
		var _el$46 = _tmpl$20();
		_$effect(() => {
			return styles[foo()];
		}, (_v$, _$p) => {
			_$className(_el$46, _v$, _$p);
		});
		return _el$46;
	})()
];
const template35 = (() => {
	var _el$47 = _tmpl$4();
	{
		var _ref$4 = a().b.c;
		typeof _ref$4 === "function" || Array.isArray(_ref$4) ? _$ref(() => {
			return _ref$4;
		}, _el$47) : a().b.c = _el$47;
	}
	return _el$47;
})();
const template36 = (() => {
	var _el$48 = _tmpl$4();
	{
		var _ref$5 = a().b?.c;
		(typeof _ref$5 === "function" || Array.isArray(_ref$5)) && _$ref(() => {
			return _ref$5;
		}, _el$48);
	}
	return _el$48;
})();
const template37 = (() => {
	var _el$49 = _tmpl$4();
	{
		var _ref$6 = a() ? b : c;
		(typeof _ref$6 === "function" || Array.isArray(_ref$6)) && _$ref(() => {
			return _ref$6;
		}, _el$49);
	}
	return _el$49;
})();
const template38 = (() => {
	var _el$50 = _tmpl$4();
	{
		var _ref$7 = a() ?? b;
		(typeof _ref$7 === "function" || Array.isArray(_ref$7)) && _$ref(() => {
			return _ref$7;
		}, _el$50);
	}
	return _el$50;
})();
const template39 = _tmpl$21();
const template40 = (() => {
	var _el$52 = _tmpl$4();
	_$effect(() => {
		return a();
	}, (_v$) => {
		_$setStyleProperty(_el$52, "color", _v$);
	});
	return _el$52;
})();
const template41 = (() => {
	var _el$53 = _tmpl$22();
	var _el$54 = _el$53.firstChild;
	var _el$55 = _el$53.firstChild.nextSibling;
	_$effect(() => {
		return state.color;
	}, (_v$) => {
		_el$53.value = _v$;
	});
	_$effect(() => {
		return Color.Red;
	}, (_v$) => {
		_el$54.value = _v$;
	});
	_$effect(() => {
		return Color.Blue;
	}, (_v$) => {
		_el$55.value = _v$;
	});
	return _el$53;
})();
const template42 = _tmpl$23();
const template43 = _tmpl$24();
const template44 = _tmpl$25();
const template45 = _tmpl$26();
const template46 = _tmpl$27();
const template47 = _tmpl$28();
const template48 = _tmpl$29();
const template49 = _tmpl$30();
const template50 = _tmpl$31();
const template51 = (() => {
	var _el$65 = _tmpl$4();
	{
		var _ref$8 = binding;
		typeof _ref$8 === "function" || Array.isArray(_ref$8) ? _$ref(() => {
			return _ref$8;
		}, _el$65) : binding = _el$65;
	}
	return _el$65;
})();
const template52 = (() => {
	var _el$66 = _tmpl$4();
	{
		var _ref$9 = binding.prop;
		typeof _ref$9 === "function" || Array.isArray(_ref$9) ? _$ref(() => {
			return _ref$9;
		}, _el$66) : binding.prop = _el$66;
	}
	return _el$66;
})();
const template53 = (() => {
	var _el$67 = _tmpl$4();
	{
		var _ref$10 = refFn;
		typeof _ref$10 === "function" || Array.isArray(_ref$10) ? _$ref(() => {
			return _ref$10;
		}, _el$67) : refFn = _el$67;
	}
	return _el$67;
})();
const template54 = (() => {
	var _el$68 = _tmpl$4();
	{
		var _ref$11 = refConst;
		typeof _ref$11 === "function" || Array.isArray(_ref$11) ? _$ref(() => {
			return _ref$11;
		}, _el$68) : refConst = _el$68;
	}
	return _el$68;
})();
const template55 = (() => {
	var _el$69 = _tmpl$4();
	{
		var _ref$12 = refUnknown;
		typeof _ref$12 === "function" || Array.isArray(_ref$12) ? _$ref(() => {
			return _ref$12;
		}, _el$69) : refUnknown = _el$69;
	}
	return _el$69;
})();
const template56 = _tmpl$32();
const template57 = _tmpl$33();
const template58 = (() => {
	var _el$72 = _tmpl$4();
	_el$72.true = true;
	_el$72.false = false;
	return _el$72;
})();
const template59 = _tmpl$34();
const template60 = (() => {
	var _el$74 = _tmpl$35();
	_$setAttribute(_el$74, "i", undefined);
	_$setAttribute(_el$74, "j", null);
	_$setAttribute(_el$74, "k", void 0);
	return _el$74;
})();
var _el$ = _$createElement2("math", { display: "block" });
var _el$2 = _$createElement2("mrow");
_$insertNode2(_el$, _el$2);
const template61 = _el$;
var _el$3 = _$createElement2("mrow");
var _el$4 = _$createElement2("mi");
_$insertNode2(_el$4, _$createTextNode2("x"));
_$insertNode2(_el$3, _el$4);
var _el$5 = _$createElement2("mo");
_$insertNode2(_el$5, _$createTextNode2("="));
_$insertNode2(_el$3, _el$5);
const template62 = _el$3;
const template63 = _tmpl$36();
const template64 = _tmpl$37();
const template65 = _tmpl$38();
const template66 = (() => {
	var _el$78 = _tmpl$38();
	_$effect(() => {
		return signal();
	}, (_v$) => {
		_$setStyleProperty(_el$78, "border", _v$);
	});
	return _el$78;
})();
const template67 = (() => {
	var _el$79 = _tmpl$38();
	_$setStyleProperty(_el$79, "border", somevalue);
	return _el$79;
})();
const template68 = (() => {
	var _el$80 = _tmpl$38();
	_$effect(() => {
		return some.access;
	}, (_v$) => {
		_$setStyleProperty(_el$80, "border", _v$);
	});
	return _el$80;
})();
const template69 = _tmpl$38();
const template70 = (() => {
	var _el$82 = _tmpl$39();
	_$setAttribute(_el$82, "playsinline", value);
	return _el$82;
})();
const template71 = _tmpl$40();
const template72 = _tmpl$39();
const template73 = _tmpl$41();
const template74 = _tmpl$42();
const template75 = (() => {
	var _el$87 = _tmpl$39();
	_el$87.poster = "1.jpg";
	return _el$87;
})();
const template76 = (() => {
	var _el$88 = _tmpl$43();
	var _el$89 = _el$88.firstChild;
	_el$89.poster = "1.jpg";
	return _el$88;
})();
// STATIC TESTS
const template77 = (() => {
	var _el$90 = _tmpl$4();
	_$setStyleProperty(_el$90, "width", props.width);
	_$setStyleProperty(_el$90, "height", props.height);
	return _el$90;
})();
const template78 = (() => {
	var _el$91 = _tmpl$4();
	_$setStyleProperty(_el$91, "width", props.width);
	_$setStyleProperty(_el$91, "height", props.height);
	_$effect(() => {
		return color();
	}, (_v$) => {
		_$setAttribute(_el$91, "something", _v$);
	});
	return _el$91;
})();
const template79 = (() => {
	var _el$92 = _tmpl$4();
	_$setStyleProperty(_el$92, "width", props.width);
	_$setStyleProperty(
		_el$92,
		"height",
		/* @static */
		props.height
	);
	_$setAttribute(
		_el$92,
		"something",
		/*@static*/
		color()
	);
	return _el$92;
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
const template80 = (() => {
	var _el$93 = _tmpl$4();
	_$spread(_el$93, propsSpread, false);
	return _el$93;
})();
const template81 = (() => {
	var _el$94 = _tmpl$4();
	_$spread(_el$94, propsSpread, false);
	return _el$94;
})();
const template82 = (() => {
	var _el$95 = _tmpl$4();
	_$spread(_el$95, _$mergeProps(propsSpread, {
		get "data-dynamic"() {
			return color();
		},
		"data-static": color()
	}), false);
	return _el$95;
})();
const template83 = (() => {
	var _el$96 = _tmpl$4();
	_$spread(_el$96, _$mergeProps(propsSpread, {
		get "data-dynamic"() {
			return color();
		},
		"data-static": color()
	}), false);
	return _el$96;
})();
const template84 = (() => {
	var _el$97 = _tmpl$4();
	_$spread(_el$97, _$mergeProps(propsSpread1, propsSpread2, propsSpread3, {
		get "data-dynamic"() {
			return color();
		},
		"data-static": color()
	}), false);
	return _el$97;
})();
// STATIC PROPERTY OF OBJECT ACCESS
// https://github.com/ryansolid/dom-expressions/issues/252#issuecomment-1572220563
const styleProp = { style: {
	width: props.width,
	height: props.height
} };
const template85 = (() => {
	var _el$98 = _tmpl$4();
	_$style(
		_el$98,
		/* @static */
		styleProp.style
	);
	return _el$98;
})();
const template86 = (() => {
	var _el$99 = _tmpl$4();
	_$effect(() => {
		return styleProp.style;
	}, (_v$, _$p) => {
		_$style(_el$99, _v$, _$p);
	});
	return _el$99;
})();
const style = {
	background: "red",
	border: "solid black " + count() + "px"
};
const template87 = (() => {
	var _el$100 = _tmpl$44();
	_$effect(() => {
		return count();
	}, (_v$) => {
		_$setAttribute(_el$100, "aria-label", _v$);
	});
	_$style(_el$100, style);
	_$className(_el$100, style);
	_$insert(_el$100, count);
	return _el$100;
})();
const template88 = (() => {
	var _el$101 = _tmpl$44();
	_$effect(() => {
		return count();
	}, (_v$) => {
		_$setAttribute(_el$101, "aria-label", _v$);
	});
	_$style(
		_el$101,
		/* @static*/
		style
	);
	_$className(
		_el$101,
		/* @static*/
		style
	);
	_$insert(_el$101, count);
	return _el$101;
})();
_$delegateEvents(["click", "input"]);
