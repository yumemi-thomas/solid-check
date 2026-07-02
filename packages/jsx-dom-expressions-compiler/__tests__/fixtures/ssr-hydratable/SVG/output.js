import { escape as _$escape } from "r-server";
import { ssr as _$ssr } from "r-server";
import { ssrHydrationKey as _$ssrHydrationKey } from "r-server";
import { ssrElement as _$ssrElement } from "r-server";
const template = _$ssr(["<svg", " width=\"400\" height=\"180\"><rect stroke-width=\"2\" x=\"50\" y=\"20\" rx=\"20\" ry=\"20\" width=\"150\" height=\"150\" style=\"fill:red;stroke:black;stroke-width:5;opacity:0.5\"></rect><linearGradient gradientTransform=\"rotate(25)\"><stop offset=\"0%\"></stop></linearGradient></svg>"], _$ssrHydrationKey());
const template2 = _$ssr([
	"<svg",
	" width=\"400\" height=\"180\"><rect class=\"",
	"\" stroke-width=\"",
	"\" x=\"",
	"\" y=\"",
	"\" rx=\"20\" ry=\"20\" width=\"150\" height=\"150\" style=\"",
	"\"></rect></svg>"
], _$ssrHydrationKey(), _$escape(state.name, true), _$escape(state.width, true), _$escape(state.x, true), _$escape(state.y, true), _$escape({
	fill: "red",
	stroke: "black",
	"stroke-width": props.stroke,
	opacity: .5
}, true));
const template3 = _$ssr([
	"<svg",
	" width=\"400\" height=\"180\">",
	"</svg>"
], _$ssrHydrationKey(), _$ssrElement("rect", props, undefined, false));
const template4 = _$ssr(["<rect", " x=\"50\" y=\"20\" width=\"150\" height=\"150\"></rect>"], _$ssrHydrationKey());
const template5 = _$ssr(["<rect", " x=\"50\" y=\"20\" width=\"150\" height=\"150\"></rect>"], _$ssrHydrationKey());
const template6 = Component({ get children() {
	return _$ssr(["<rect", " x=\"50\" y=\"20\" width=\"150\" height=\"150\"></rect>"], _$ssrHydrationKey());
} });
const template7 = _$ssr([
	"<svg",
	" viewBox=\"0 0 160 40\" xmlns=\"http://www.w3.org/2000/svg\"><a xlink:href=\"",
	"\"><text x=\"10\" y=\"25\">MDN Web Docs</text></a></svg>"
], _$ssrHydrationKey(), _$escape(url, true));
const template8 = _$ssr([
	"<svg",
	" viewBox=\"0 0 160 40\" xmlns=\"http://www.w3.org/2000/svg\"><text x=\"10\" y=\"25\" textContent=\"",
	"\"></text></svg>"
], _$ssrHydrationKey(), _$escape(text, true));
