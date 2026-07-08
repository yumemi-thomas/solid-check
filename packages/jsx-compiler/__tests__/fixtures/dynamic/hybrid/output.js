import { createComponent as _$createComponent2 } from "r-custom";
import { spread as _$spread2 } from "r-custom";
import { insert as _$insert2 } from "r-custom";
import { createElement as _$createElement2 } from "r-custom";
import { template as _$template } from "r-dom";
import { insert as _$insert } from "r-dom";
import { createComponent as _$createComponent } from "r-dom";
import { spread as _$spread } from "r-dom";
import { ref as _$ref } from "r-dom";
import { effect as _$effect } from "r-dom";
import { setAttribute as _$setAttribute } from "r-dom";
var _tmpl$ = /* @__PURE__ */ _$template(`<div>Hello`);
var _tmpl$2 = /* @__PURE__ */ _$template(`<mesh scale=2>`);
var _tmpl$3 = /* @__PURE__ */ _$template(`<pointLight>`);
var _tmpl$4 = /* @__PURE__ */ _$template(`<button>`);
var _tmpl$5 = /* @__PURE__ */ _$template(`<div><div></div>`);
var _tmpl$6 = /* @__PURE__ */ _$template(`<mesh scale=2><pointLight>`);
var _tmpl$7 = /* @__PURE__ */ _$template(`<div><button>`);
var _tmpl$8 = /* @__PURE__ */ _$template(`<div>`);
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
		_$effect(() => {
			return <div backgroundColor={s() ? "red" : "green"} />;
		}, (_v$) => {
			_$setAttribute(_el$, "element", _v$);
		});
		_$insert(_el$, () => {
			return props.name;
		}, null);
		return _el$;
	})(), (() => {
		var _el$2 = _tmpl$5();
		var _el$3 = _el$2.firstChild;
		{
			var _ref$2 = set;
			typeof _ref$2 === "function" || Array.isArray(_ref$2) ? _$ref(() => {
				return _ref$2;
			}, _el$3) : set = _el$3;
		}
		_$insert(_el$3, () => {
			return props.children;
		});
		_$insert(_el$2, _$createComponent(Canvas, { get children() {
			var _el$4 = _tmpl$2();
			_$effect(() => {
				return [
					0,
					0,
					0
				];
			}, (_v$) => {
				_$setAttribute(_el$4, "position", _v$);
			});
			_$effect(() => {
				return <boxBufferGeometry args={[
					0,
					1,
					2
				]} />;
			}, (_v$) => {
				_$setAttribute(_el$4, "geometry", _v$);
			});
			_$effect(() => {
				return <basicMaterial alpha={0} color={s() ? "red" : "green"} />;
			}, (_v$) => {
				_$setAttribute(_el$4, "material", _v$);
			});
			return [
				_el$4,
				_tmpl$3(),
				_$createComponent(HTML, { get children() {
					var _el$6 = _tmpl$();
					{
						var _ref$3 = props.ref;
						typeof _ref$3 === "function" || Array.isArray(_ref$3) ? _$ref(() => {
							return _ref$3;
						}, _el$6) : props.ref = _el$6;
					}
					_$effect(() => {
						return <div backgroundColor={s() ? "red" : "green"} />;
					}, (_v$) => {
						_$setAttribute(_el$6, "element", _v$);
					});
					_$insert(_el$6, () => {
						return props.name;
					}, null);
					return [_el$6, _tmpl$4()];
				} })
			];
		} }), null);
		return _el$2;
	})()];
};
const Component = (props) => {
	return (() => {
		var _el$8 = _tmpl$8();
		_$insert(_el$8, () => {
			return props.three ? (() => {
				var _el$9 = _tmpl$6();
				_$effect(() => {
					return [
						0,
						0,
						0
					];
				}, (_v$) => {
					_$setAttribute(_el$9, "position", _v$);
				});
				_$effect(() => {
					return <boxBufferGeometry args={[
						0,
						1,
						2
					]} />;
				}, (_v$) => {
					_$setAttribute(_el$9, "geometry", _v$);
				});
				_$effect(() => {
					return <basicMaterial alpha={0} color={s() ? "red" : "green"} />;
				}, (_v$) => {
					_$setAttribute(_el$9, "material", _v$);
				});
				return _el$9;
			})() : _tmpl$7();
		});
		return _el$8;
	})();
};
const Mesh = (props) => {
	return (() => {
		var _el$ = _$createElement2("group");
		_$spread2(_el$, props, true);
		_$insert2(_el$, [(() => {
			var _el$2 = _$createElement2("group");
			_$insert2(_el$2, a ? <mesh /> : <instancedMesh />);
			return _el$2;
		})(), _$createComponent2(HTML, { get children() {
			return (() => {
				var _el$11 = _tmpl$8();
				_$spread(_el$11, props, true);
				_$insert(_el$11, () => {
					return b ? _tmpl$8() : _tmpl$4();
				});
				return _el$11;
			})();
		} })]);
		return _el$;
	})();
};
