import { createTextNode as _$createTextNode } from "r-custom";
import { createComponent as _$createComponent } from "r-custom";
import { insertNode as _$insertNode } from "r-custom";
import { setProp as _$setProp } from "r-custom";
import { createElement as _$createElement } from "r-custom";
const multiStatic = [(() => {
	var _el$ = _$createElement("div");
	_$insertNode(_el$, _$createTextNode("First"));
	return _el$;
})(), (() => {
	var _el$2 = _$createElement("div");
	_$insertNode(_el$2, _$createTextNode("Last"));
	return _el$2;
})()];
const multiExpression = [
	(() => {
		var _el$3 = _$createElement("div");
		_$insertNode(_el$3, _$createTextNode("First"));
		return _el$3;
	})(),
	inserted,
	(() => {
		var _el$4 = _$createElement("div");
		_$insertNode(_el$4, _$createTextNode("Last"));
		return _el$4;
	})(),
	"After"
];
const multiDynamic = [
	(() => {
		var _el$5 = _$createElement("div");
		_$setProp(_el$5, "id", state.first);
		_$insertNode(_el$5, _$createTextNode("First"));
		return _el$5;
	})(),
	state.inserted,
	(() => {
		var _el$6 = _$createElement("div");
		_$setProp(_el$6, "id", state.last);
		_$insertNode(_el$6, _$createTextNode("Last"));
		return _el$6;
	})(),
	"After"
];
const singleExpression = inserted;
const singleDynamic = inserted();
const firstStatic = [inserted, (() => {
	var _el$7 = _$createElement("div");
	return _el$7;
})()];
const firstDynamic = [inserted(), (() => {
	var _el$8 = _$createElement("div");
	return _el$8;
})()];
const firstComponent = [_$createComponent(Component, {}), (() => {
	var _el$9 = _$createElement("div");
	return _el$9;
})()];
const lastStatic = [(() => {
	var _el$10 = _$createElement("div");
	return _el$10;
})(), inserted];
const lastDynamic = [(() => {
	var _el$11 = _$createElement("div");
	return _el$11;
})(), inserted()];
const lastComponent = [(() => {
	var _el$12 = _$createElement("div");
	return _el$12;
})(), _$createComponent(Component, {})];
const spaces = [
	(() => {
		var _el$13 = _$createElement("span");
		_$insertNode(_el$13, _$createTextNode("1"));
		return _el$13;
	})(),
	" ",
	(() => {
		var _el$14 = _$createElement("span");
		_$insertNode(_el$14, _$createTextNode("2"));
		return _el$14;
	})(),
	" ",
	(() => {
		var _el$15 = _$createElement("span");
		_$insertNode(_el$15, _$createTextNode("3"));
		return _el$15;
	})()
];
const multiLineTrailing = [
	(() => {
		var _el$16 = _$createElement("span");
		_$insertNode(_el$16, _$createTextNode("1"));
		return _el$16;
	})(),
	(() => {
		var _el$17 = _$createElement("span");
		_$insertNode(_el$17, _$createTextNode("2"));
		return _el$17;
	})(),
	(() => {
		var _el$18 = _$createElement("span");
		_$insertNode(_el$18, _$createTextNode("3"));
		return _el$18;
	})()
];
