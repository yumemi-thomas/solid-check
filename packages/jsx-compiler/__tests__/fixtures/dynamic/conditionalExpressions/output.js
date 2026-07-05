import { createComponent as _$createComponent2 } from "r-custom";
import { template as _$template } from "r-dom";
import { insert as _$insert } from "r-dom";
import { memo as _$memo } from "r-dom";
import { createComponent as _$createComponent } from "r-dom";
import { effect as _$effect } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<div>`);
var _tmpl$2 = /* @__PURE__ */ _$template(`<div>Output`);
const template1 = (() => {
	var _el$ = _tmpl$();
	_$insert(_el$, simple);
	return _el$;
})();
const template2 = (() => {
	var _el$2 = _tmpl$();
	_$insert(_el$2, () => {
		return state.dynamic;
	});
	return _el$2;
})();
const template3 = (() => {
	var _el$3 = _tmpl$();
	_$insert(_el$3, simple ? good : bad);
	return _el$3;
})();
const template4 = (() => {
	var _el$4 = _tmpl$();
	_$insert(_el$4, () => {
		return simple ? good() : bad;
	});
	return _el$4;
})();
const template4a = (() => {
	var _el$5 = _tmpl$();
	_$insert(_el$5, () => {
		return simple ? good.good : bad;
	});
	return _el$5;
})();
const template5 = (() => {
	var _el$6 = _tmpl$();
	_$insert(_el$6, (() => {
		var _c$ = _$memo(() => {
			return !!state.dynamic;
		});
		return () => {
			return _c$() ? good() : bad;
		};
	})());
	return _el$6;
})();
const template5a = (() => {
	var _el$7 = _tmpl$();
	_$insert(_el$7, (() => {
		var _c$2 = _$memo(() => {
			return !!state.dynamic;
		});
		return () => {
			return _c$2() ? good.gppd : bad;
		};
	})());
	return _el$7;
})();
const template6 = (() => {
	var _el$8 = _tmpl$();
	_$insert(_el$8, (() => {
		var _c$3 = _$memo(() => {
			return !!state.dynamic;
		});
		return () => {
			return _c$3() ? good() : state.dynamic;
		};
	})());
	return _el$8;
})();
const template6a = (() => {
	var _el$9 = _tmpl$();
	_$insert(_el$9, (() => {
		var _c$4 = _$memo(() => {
			return !!state.dynamic;
		});
		return () => {
			return _c$4() ? good.good : state.dynamic;
		};
	})());
	return _el$9;
})();
const template7 = (() => {
	var _el$10 = _tmpl$();
	_$insert(_el$10, state.count > 5 ? state.dynamic ? best : good() : bad);
	return _el$10;
})();
const template7a = (() => {
	var _el$11 = _tmpl$();
	_$insert(_el$11, state.count > 5 ? state.dynamic ? best : good.good : bad);
	return _el$11;
})();
const template8 = (() => {
	var _el$12 = _tmpl$();
	_$insert(_el$12, (() => {
		var _c$5 = _$memo(() => {
			return !!(state.dynamic && state.something);
		});
		return () => {
			return _c$5() ? good() : state.dynamic && state.something;
		};
	})());
	return _el$12;
})();
const template8a = (() => {
	var _el$13 = _tmpl$();
	_$insert(_el$13, (() => {
		var _c$6 = _$memo(() => {
			return !!(state.dynamic && state.something);
		});
		return () => {
			return _c$6() ? good.good : state.dynamic && state.something;
		};
	})());
	return _el$13;
})();
const template9 = (() => {
	var _el$14 = _tmpl$();
	_$insert(_el$14, state.dynamic && good() || bad);
	return _el$14;
})();
const template9a = (() => {
	var _el$15 = _tmpl$();
	_$insert(_el$15, state.dynamic && good.good || bad);
	return _el$15;
})();
const template10 = (() => {
	var _el$16 = _tmpl$();
	_$insert(_el$16, (() => {
		var _c$7 = _$memo(() => {
			return !!state.a;
		});
		return () => {
			return _c$7() ? "a" : _$memo(() => {
				return !!state.b;
			})() ? "b" : state.c ? "c" : "fallback";
		};
	})());
	return _el$16;
})();
const template11 = (() => {
	var _el$17 = _tmpl$();
	_$insert(_el$17, (() => {
		var _c$8 = _$memo(() => {
			return !!state.a;
		});
		return () => {
			return _c$8() ? a() : _$memo(() => {
				return !!state.b;
			})() ? b() : state.c ? "c" : "fallback";
		};
	})());
	return _el$17;
})();
const template11a = (() => {
	var _el$18 = _tmpl$();
	_$insert(_el$18, (() => {
		var _c$9 = _$memo(() => {
			return !!state.a;
		});
		return () => {
			return _c$9() ? a.a : _$memo(() => {
				return !!state.b;
			})() ? b.b : state.c ? "c" : "fallback";
		};
	})());
	return _el$18;
})();
const template12 = _$createComponent2(Comp, { render: state.dynamic ? good() : bad });
const template12a = _$createComponent2(Comp, { render: state.dynamic ? good.good : bad });
// no dynamic predicate
const template13 = _$createComponent2(Comp, { render: state.dynamic ? good : bad });
const template14 = _$createComponent2(Comp, { render: state.dynamic && good() });
const template14a = _$createComponent2(Comp, { render: state.dynamic && good.good });
// no dynamic predicate
const template15 = _$createComponent2(Comp, { render: state.dynamic && good });
const template16 = _$createComponent2(Comp, { render: state.dynamic || good() });
const template16a = _$createComponent2(Comp, { render: state.dynamic || good.good });
const template17 = _$createComponent2(Comp, { render: state.dynamic ? _$createComponent2(Comp, {}) : _$createComponent2(Comp, {}) });
const template18 = _$createComponent2(Comp, { get children() {
	return state.dynamic ? _$createComponent2(Comp, {}) : _$createComponent2(Comp, {});
} });
const template19 = (() => {
	var _el$19 = _tmpl$();
	_$effect(() => {
		return state.dynamic ? <Comp /> : <Comp />;
	}, (_v$) => {
		_el$19.innerHTML = _v$;
	});
	return _el$19;
})();
const template20 = (() => {
	var _el$20 = _tmpl$();
	_$insert(_el$20, (() => {
		var _c$10 = _$memo(() => {
			return !!state.dynamic;
		});
		return () => {
			return _c$10() ? _$createComponent(Comp, {}) : _$createComponent(Comp, {});
		};
	})());
	return _el$20;
})();
const template21 = _$createComponent2(Comp, { render: state?.dynamic ? "a" : "b" });
const template22 = _$createComponent2(Comp, { get children() {
	return state?.dynamic ? "a" : "b";
} });
const template23 = (() => {
	var _el$21 = _tmpl$();
	_$effect(() => {
		return state?.dynamic ? "a" : "b";
	}, (_v$) => {
		_el$21.innerHTML = _v$;
	});
	return _el$21;
})();
const template24 = (() => {
	var _el$22 = _tmpl$();
	_$insert(_el$22, () => {
		return state?.dynamic ? "a" : "b";
	});
	return _el$22;
})();
const template25 = _$createComponent2(Comp, { render: state.dynamic ?? _$createComponent2(Comp, {}) });
const template26 = _$createComponent2(Comp, { get children() {
	return state.dynamic ?? _$createComponent2(Comp, {});
} });
const template27 = (() => {
	var _el$23 = _tmpl$();
	_$effect(() => {
		return state.dynamic ?? <Comp />;
	}, (_v$) => {
		_el$23.innerHTML = _v$;
	});
	return _el$23;
})();
const template28 = (() => {
	var _el$24 = _tmpl$();
	_$insert(_el$24, () => {
		return state.dynamic ?? _$createComponent(Comp, {});
	});
	return _el$24;
})();
const template29 = (() => {
	var _el$25 = _tmpl$();
	_$insert(_el$25, () => {
		return (thing() && thing1()) ?? thing2() ?? thing3();
	});
	return _el$25;
})();
const template29a = (() => {
	var _el$26 = _tmpl$();
	_$insert(_el$26, () => {
		return (thing.thing && thing1.thing1) ?? thing2.thing2 ?? thing3.thing3;
	});
	return _el$26;
})();
const template30 = (() => {
	var _el$27 = _tmpl$();
	_$insert(_el$27, () => {
		return thing() || thing1() || thing2();
	});
	return _el$27;
})();
const template30a = (() => {
	var _el$28 = _tmpl$();
	_$insert(_el$28, () => {
		return thing.thing || thing1.thing1 || thing2.thing2;
	});
	return _el$28;
})();
const template31 = _$createComponent2(Comp, { value: count() ? count() ? count() : count() : count() });
const template31a = _$createComponent2(Comp, { value: count.count ? count.count ? count.count : count.count : count.count });
const template32 = (() => {
	var _el$29 = _tmpl$();
	_$insert(_el$29, () => {
		return something?.();
	});
	return _el$29;
})();
const template33 = _$createComponent2(Comp, { get children() {
	return something?.();
} });
const template34 = simple ? good : bad;
const template35 = simple ? good() : bad;
const template35a = simple ? good.good : bad;
const template36 = state.dynamic ? good() : bad;
const template36a = state.dynamic ? good.good : bad;
const template37 = state.dynamic && good();
const template37a = state.dynamic && good.good;
const template38 = state.count > 5 ? state.dynamic ? best : good() : bad;
const template38a = state.count > 5 ? state.dynamic ? best : good.good : bad;
const template39 = state.dynamic && state.something && good();
const template39a = state.dynamic && state.something && good.good;
const template40 = state.dynamic && good() || bad;
const template40a = state.dynamic && good.good || bad;
const template41 = state.a ? "a" : state.b ? "b" : state.c ? "c" : "fallback";
const template42 = state.a ? a() : state.b ? b() : state.c ? "c" : "fallback";
const template42a = state.a ? a.a : state.b ? b.b : state.c ? "c" : "fallback";
const template43 = obj1.prop ? obj2.prop ? _tmpl$2() : null : null;
