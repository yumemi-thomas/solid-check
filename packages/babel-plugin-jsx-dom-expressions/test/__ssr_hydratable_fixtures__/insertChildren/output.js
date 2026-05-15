import { mergeProps as _$mergeProps } from "r-server";
import { ssrElement as _$ssrElement } from "r-server";
import { escape as _$escape } from "r-server";
import { ssr as _$ssr } from "r-server";
import { ssrHydrationKey as _$ssrHydrationKey } from "r-server";
var _v$19;
var _tmpl$ = ["<div", "></div>"],
  _tmpl$2 = ["<module", ">", "</module>"],
  _tmpl$3 = ["<module", ">Hello</module>"],
  _tmpl$4 = ["<module", ">Hi <!--$-->", "<!--/--></module>"],
  _tmpl$5 = ["<div", ">Test 1</div>"],
  _tmpl$6 = ["<div", ">", "</div>"];
var _v$ = _$ssrHydrationKey();
const children = _$ssr(_tmpl$, _v$);
const dynamic = {
  children
};
const template = Module({
  children: children
});
var _v$2 = _$ssrHydrationKey(),
  _v$3 = _$escape(children);
const template2 = _$ssr(_tmpl$2, _v$2, _v$3);
var _v$4 = _$ssrHydrationKey();
const template3 = _$ssr(_tmpl$3, _v$4);
var _v$5 = _$ssrHydrationKey(),
  _v$6 = _$escape(Hello({}));
const template4 = _$ssr(_tmpl$2, _v$5, _v$6);
var _v$7 = _$ssrHydrationKey(),
  _v$8 = () => _$escape(dynamic.children);
const template5 = _$ssr(_tmpl$2, _v$7, _v$8);
const template6 = Module({
  get children() {
    return dynamic.children;
  }
});
const template7 = _$ssrElement("module", dynamic, undefined, true);
const template8 = _$ssrElement("module", dynamic, () => "Hello", true);
const template9 = _$ssrElement("module", dynamic, () => () => _$escape(dynamic.children), true);
const template10 = Module(
  _$mergeProps(dynamic, {
    children: "Hello"
  })
);
var _v$9 = _$ssrHydrationKey(),
  _v$0 = _$escape(/*@static*/ state.children);
const template11 = _$ssr(_tmpl$2, _v$9, _v$0);
const template12 = Module({
  children: state.children
});
var _v$1 = _$ssrHydrationKey(),
  _v$10 = _$escape(children);
const template13 = _$ssr(_tmpl$2, _v$1, _v$10);
const template14 = Module({
  children: children
});
var _v$11 = _$ssrHydrationKey(),
  _v$12 = () => _$escape(dynamic.children);
const template15 = _$ssr(_tmpl$2, _v$11, _v$12);
const template16 = Module({
  get children() {
    return dynamic.children;
  }
});
var _v$13 = _$ssrHydrationKey(),
  _v$14 = _$escape(children);
const template18 = _$ssr(_tmpl$4, _v$13, _v$14);
const template19 = Module({
  get children() {
    return ["Hi ", children];
  }
});
var _v$15 = _$ssrHydrationKey(),
  _v$16 = () => _$escape(children());
const template20 = _$ssr(_tmpl$2, _v$15, _v$16);
const template21 = Module({
  get children() {
    return children();
  }
});
var _v$17 = _$ssrHydrationKey(),
  _v$18 = () => _$escape(state.children());
const template22 = _$ssr(_tmpl$2, _v$17, _v$18);
const template23 = Module({
  get children() {
    return state.children();
  }
});
const template24 = _$ssrElement(
  "module",
  dynamic,
  () => ["Hi", "<!--$-->", () => _$escape(dynamic.children), "<!--/-->"],
  true
);
const tiles = [];
tiles.push(((_v$19 = _$ssrHydrationKey()), _$ssr(_tmpl$5, _v$19)));
var _v$20 = _$ssrHydrationKey(),
  _v$21 = _$escape(tiles);
const template25 = _$ssr(_tmpl$6, _v$20, _v$21);
var _v$22 = _$ssrHydrationKey(),
  _v$23 = () => _$escape((expression(), "static"));
const comma = _$ssr(_tmpl$6, _v$22, _v$23);
var _v$24 = _$ssrHydrationKey(),
  _v$25 = () => _$escape(children()());
const double = _$ssr(_tmpl$6, _v$24, _v$25);
