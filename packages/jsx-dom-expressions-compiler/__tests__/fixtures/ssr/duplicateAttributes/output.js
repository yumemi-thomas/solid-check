import { escape as _$escape } from "r-server";
import { ssr as _$ssr } from "r-server";
// Duplicate attributes (not just `class`) resolve to the last value.
const dynamicId = () => "dyn-id";
// Same attribute twice, both static.
const t1 = _$ssr("<div id=\"first\" id=\"second\">id</div>");
// Static then dynamic — dynamic wins.
const t2 = _$ssr(["<div title=\"static\" title=\"", "\">title</div>"], _$escape(dynamicId(), true));
// Dynamic then static — static wins.
const t3 = _$ssr(["<div data-x=\"", "\" data-x=\"fixed\">data</div>"], _$escape(dynamicId(), true));
// Namespaced (xlink:href) duplicates.
const t4 = _$ssr("<svg><use xlink:href=\"#a\" xlink:href=\"#b\"></use></svg>");
// Boolean attribute duplicated with different values.
const t5 = _$ssr("<input disabled disabled=\"false\">");
