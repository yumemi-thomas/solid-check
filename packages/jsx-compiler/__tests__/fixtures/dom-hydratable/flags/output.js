import { getNextElement as _$getNextElement } from "r-dom";
import { getNextMarker as _$getNextMarker } from "r-dom";
import { insert as _$insert } from "r-dom";
import { createComponent as _$createComponent } from "r-dom";
const template = (() => {
	var _el$ = _$getNextElement();
	var [_el$2, _el$3] = _$getNextMarker(_el$.firstChild.nextSibling.nextSibling);
	var [_el$4, _el$5] = _$getNextMarker(_el$2.nextSibling.nextSibling);
	_$insert(_el$, _$createComponent(Component, {}), _el$2, _el$3);
	_$insert(_el$, () => {
		return state.interpolation;
	}, _el$4, _el$5);
	return _el$;
})();
const template2 = _$createComponent(Component, { get children() {
	return _$getNextElement();
} });
const template3 = _$createComponent(Component, { get children() {
	return [_$getNextElement(), _$getNextElement()];
} });
const template4 = _$getNextElement();
