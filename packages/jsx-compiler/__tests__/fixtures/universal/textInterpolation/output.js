import { createTextNode as _$createTextNode } from "r-custom";
import { insert as _$insert } from "r-custom";
import { insertNode as _$insertNode } from "r-custom";
import { setProp as _$setProp } from "r-custom";
import { createElement as _$createElement } from "r-custom";
var _el$ = _$createElement("span");
_$insertNode(_el$, _$createTextNode("Hello "));
const trailing = _el$;
var _el$2 = _$createElement("span");
_$insertNode(_el$2, _$createTextNode(" John"));
const leading = _el$2;
var _el$3 = _$createElement("span");
_$insertNode(_el$3, _$createTextNode("Hello John"));
/* prettier-ignore */
const extraSpaces = _el$3;
var _el$4 = _$createElement("span");
_$insertNode(_el$4, _$createTextNode("Hello "));
_$insert(_el$4, name);
const trailingExpr = _el$4;
var _el$5 = _$createElement("span");
_$insert(_el$5, greeting);
_$insertNode(_el$5, _$createTextNode(" John"));
const leadingExpr = _el$5;
var _el$6 = _$createElement("span");
_$insert(_el$6, greeting);
_$insertNode(_el$6, _$createTextNode(" "));
_$insert(_el$6, name);
/* prettier-ignore */
const multiExpr = _el$6;
var _el$7 = _$createElement("span");
_$insertNode(_el$7, _$createTextNode(" "));
_$insert(_el$7, greeting);
_$insertNode(_el$7, _$createTextNode(" "));
_$insert(_el$7, name);
_$insertNode(_el$7, _$createTextNode(" "));
/* prettier-ignore */
const multiExprSpaced = _el$7;
var _el$8 = _$createElement("span");
_$insertNode(_el$8, _$createTextNode(" "));
_$insert(_el$8, greeting);
_$insert(_el$8, name);
_$insertNode(_el$8, _$createTextNode(" "));
/* prettier-ignore */
const multiExprTogether = _el$8;
var _el$9 = _$createElement("span");
_$insertNode(_el$9, _$createTextNode("Hello"));
/* prettier-ignore */
const multiLine = _el$9;
var _el$10 = _$createElement("span");
_$insertNode(_el$10, _$createTextNode("Hello John"));
/* prettier-ignore */
const multiLineTrailingSpace = _el$10;
var _el$11 = _$createElement("span");
_$insertNode(_el$11, _$createTextNode("Hello John"));
/* prettier-ignore */
const multiLineNoTrailingSpace = _el$11;
var _el$12 = _$createElement("span");
_$insertNode(_el$12, _$createTextNode("&nbsp;&lt;Hi&gt;&nbsp;"));
/* prettier-ignore */
const escape = _el$12;
var _el$13 = _$createElement("span");
_$insertNode(_el$13, _$createTextNode("Hi"));
_$insertNode(_el$13, _$createTextNode("<script>alert();<\/script>"));
/* prettier-ignore */
const injection = _el$13;
let value = "World";
var _el$14 = _$createElement("span");
_$insertNode(_el$14, _$createTextNode("Hello "));
_$insert(_el$14, value + "!");
const evaluated = _el$14;
let number = 4 + 5;
var _el$15 = _$createElement("span");
_$insertNode(_el$15, _$createTextNode("4 + 5 = "));
_$insert(_el$15, number);
const evaluatedNonString = _el$15;
var _el$16 = _$createElement("div");
_$insert(_el$16, s);
_$insertNode(_el$16, _$createTextNode("\n"));
_$insertNode(_el$16, _$createTextNode("d"));
const newLineLiteral = _el$16;
var _el$17 = _$createElement("div");
_$insert(_el$17, expr);
const trailingSpace = _el$17;
var _el$18 = _$createElement("span");
_$insertNode(_el$18, _$createTextNode(" "));
_$insert(_el$18, expr);
const leadingSpaceElement = _el$18;
var _el$19 = _$createElement("span");
_$insert(_el$19, expr);
_$insertNode(_el$19, _$createTextNode(" "));
const trailingSpaceElement = _el$19;
var _el$20 = _$createElement("div");
_$setProp(_el$20, "normal", "Search…");
_$setProp(_el$20, "title", "Search&hellip;");
const escapeAttribute = _el$20;
