import { scope as _$scope } from "r-server";
import { escape as _$escape } from "r-server";
import { ssr as _$ssr } from "r-server";
import { ssrHydrationKey as _$ssrHydrationKey } from "r-server";
const trailing = _$ssr(["<span", ">Hello </span>"], _$ssrHydrationKey());
const leading = _$ssr(["<span", "> John</span>"], _$ssrHydrationKey());
/* prettier-ignore */
const extraSpaces = _$ssr(["<span", ">Hello John</span>"], _$ssrHydrationKey());
const trailingExpr = _$ssr([
	"<span",
	">Hello ",
	"</span>"
], _$ssrHydrationKey(), _$escape(name));
const leadingExpr = _$ssr([
	"<span",
	">",
	" John</span>"
], _$ssrHydrationKey(), _$escape(greeting));
/* prettier-ignore */
const multiExpr = _$ssr([
	"<span",
	">",
	" ",
	"</span>"
], _$ssrHydrationKey(), _$escape(greeting), _$escape(name));
/* prettier-ignore */
const multiExprSpaced = _$ssr([
	"<span",
	"> ",
	" ",
	" </span>"
], _$ssrHydrationKey(), _$escape(greeting), _$escape(name));
/* prettier-ignore */
const multiExprTogether = _$ssr([
	"<span",
	"> ",
	"",
	" </span>"
], _$ssrHydrationKey(), _$escape(greeting), _$escape(name));
/* prettier-ignore */
const multiLine = _$ssr(["<span", ">Hello</span>"], _$ssrHydrationKey());
/* prettier-ignore */
const multiLineTrailingSpace = _$ssr(["<span", ">Hello John</span>"], _$ssrHydrationKey());
/* prettier-ignore */
const multiLineNoTrailingSpace = _$ssr(["<span", ">Hello John</span>"], _$ssrHydrationKey());
/* prettier-ignore */
const escape = _$ssr(["<span", ">&nbsp;&lt;Hi&gt;&nbsp;</span>"], _$ssrHydrationKey());
/* prettier-ignore */
const escape2 = Comp({ children: "&nbsp;&lt;Hi&gt;&nbsp;" });
/* prettier-ignore */
const escape3 = "&nbsp;&lt;Hi&gt;&nbsp;";
const injection = _$ssr(["<span", ">Hi&lt;script>alert();&lt;/script></span>"], _$ssrHydrationKey());
let value = "World";
const evaluated = _$ssr([
	"<span",
	">Hello ",
	"</span>"
], _$ssrHydrationKey(), _$escape(value + "!"));
let number = 4 + 5;
const evaluatedNonString = _$ssr([
	"<span",
	">4 + 5 = ",
	"</span>"
], _$ssrHydrationKey(), _$escape(number));
const newLineLiteral = _$ssr([
	"<div",
	">",
	"\nd</div>"
], _$ssrHydrationKey(), _$escape(s));
const trailingSpace = _$ssr([
	"<div",
	">",
	"</div>"
], _$ssrHydrationKey(), _$escape(expr));
const trailingSpaceComp = Comp({ get children() {
	return expr;
} });
const trailingSpaceFrag = expr;
const leadingSpaceElement = _$ssr([
	"<span",
	"> ",
	"</span>"
], _$ssrHydrationKey(), _$escape(expr));
const leadingSpaceComponent = Div({ get children() {
	return [" ", expr];
} });
const leadingSpaceFragment = [" ", expr];
const trailingSpaceElement = _$ssr([
	"<span",
	">",
	" </span>"
], _$ssrHydrationKey(), _$escape(expr));
const trailingSpaceComponent = Div({ get children() {
	return [expr, " "];
} });
const trailingSpaceFragment = [expr, " "];
const escapeAttribute = _$ssr(["<div", " normal=\"Search&amp;hellip;\" title=\"Search&amp;hellip;\"></div>"], _$ssrHydrationKey());
const escapeCompAttribute = Div({
	normal: "Search…",
	title: "Search&hellip;"
});
const lastElementExpression = _$ssr([
	"<div",
	"><div></div>",
	"</div>"
], _$ssrHydrationKey(), _$scope(() => {
	return _$escape(expr());
}));
