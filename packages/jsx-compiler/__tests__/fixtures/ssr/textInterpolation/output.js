import { escape as _$escape } from "r-server";
import { ssr as _$ssr } from "r-server";
const trailing = _$ssr("<span>Hello </span>");
const leading = _$ssr("<span> John</span>");
/* prettier-ignore */
const extraSpaces = _$ssr("<span>Hello John</span>");
const trailingExpr = _$ssr(["<span>Hello ", "</span>"], _$escape(name));
const leadingExpr = _$ssr(["<span>", " John</span>"], _$escape(greeting));
/* prettier-ignore */
const multiExpr = _$ssr([
	"<span>",
	" ",
	"</span>"
], _$escape(greeting), _$escape(name));
/* prettier-ignore */
const multiExprSpaced = _$ssr([
	"<span> ",
	" ",
	" </span>"
], _$escape(greeting), _$escape(name));
/* prettier-ignore */
const multiExprTogether = _$ssr([
	"<span> ",
	"",
	" </span>"
], _$escape(greeting), _$escape(name));
/* prettier-ignore */
const multiLine = _$ssr("<span>Hello</span>");
/* prettier-ignore */
const multiLineTrailingSpace = _$ssr("<span>Hello John</span>");
/* prettier-ignore */
const multiLineNoTrailingSpace = _$ssr("<span>Hello John</span>");
/* prettier-ignore */
const escape = _$ssr("<span>&nbsp;&lt;Hi&gt;&nbsp;</span>");
/* prettier-ignore */
const injection = _$ssr("<span>Hi&lt;script>alert();&lt;/script></span>");
let value = "World";
const evaluated = _$ssr(["<span>Hello ", "</span>"], _$escape(value + "!"));
let number = 4 + 5;
const evaluatedNonString = _$ssr(["<span>4 + 5 = ", "</span>"], _$escape(number));
const newLineLiteral = _$ssr(["<div>", "\nd</div>"], _$escape(s));
const trailingSpace = _$ssr(["<div>", "</div>"], _$escape(expr));
const leadingSpaceElement = _$ssr(["<span> ", "</span>"], _$escape(expr));
const trailingSpaceElement = _$ssr(["<span>", " </span>"], _$escape(expr));
const escapeAttribute = _$ssr("<div normal=\"Search…\" title=\"Search&amp;hellip;\"></div>");
const lastElementExpression = _$ssr(["<div><div></div>", "</div>"], _$escape(expr()));
const messwithTemplates = _$ssr("<p>${blah}</p>");
