import { getNextElement as _$getNextElement } from "r-dom";
import { getNextMarker as _$getNextMarker } from "r-dom";
import { getNextMatch as _$getNextMatch } from "r-dom";
import { insert as _$insert } from "r-dom";
import { createComponent as _$createComponent } from "r-dom";
const template = (() => {
	var _el$ = _$getNextElement();
	var _el$2 = _$getNextMatch(_el$.firstChild, "head");
	var [_el$3, _el$4] = _$getNextMarker(_el$2.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling);
	var _el$5 = _$getNextMatch(_el$2.nextSibling, "body");
	var [_el$6, _el$7] = _$getNextMarker(_el$5.firstChild.nextSibling.nextSibling);
	_$insert(_el$2, _$createComponent(Assets, {}), _el$3, _el$4);
	_$insert(_el$5, _$createComponent(App, {}), _el$6, _el$7);
	return _el$;
})();
const templateHead = (() => {
	var _el$8 = _$getNextElement();
	var [_el$9, _el$10] = _$getNextMarker(_el$8.firstChild.nextSibling.nextSibling.nextSibling.nextSibling.nextSibling);
	_$insert(_el$8, _$createComponent(Assets, {}), _el$9, _el$10);
	return _el$8;
})();
const templateBody = (() => {
	var _el$11 = _$getNextElement();
	var [_el$12, _el$13] = _$getNextMarker(_el$11.firstChild.nextSibling.nextSibling);
	_$insert(_el$11, _$createComponent(App, {}), _el$12, _el$13);
	return _el$11;
})();
const templateEmptied = (() => {
	var _el$14 = _$getNextElement();
	var [_el$15, _el$16] = _$getNextMarker(_el$14.firstChild.nextSibling);
	var [_el$17, _el$18] = _$getNextMarker(_el$15.nextSibling.nextSibling);
	_$insert(_el$14, _$createComponent(Head, {}), _el$15, _el$16);
	_$insert(_el$14, _$createComponent(Body, {}), _el$17, _el$18);
	return _el$14;
})();
