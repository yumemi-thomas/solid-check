import { ssr as _$ssr } from "r-server";
import { ssrHydrationKey as _$ssrHydrationKey } from "r-server";
const template = _$ssr([
	"<html",
	"><head><title>🔥 Blazing 🔥</title><meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\"><link rel=\"stylesheet\" href=\"/styles.css\">",
	"</head><body><header><h1>Welcome to the Jungle</h1></header>",
	"<footer>The Bottom</footer></body></html>"
], _$ssrHydrationKey(), Assets({}), App({}));
const templateHead = _$ssr([
	"<head",
	"><title>🔥 Blazing 🔥</title><meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\"><link rel=\"stylesheet\" href=\"/styles.css\">",
	"</head>"
], _$ssrHydrationKey(), Assets({}));
const templateBody = _$ssr([
	"<body",
	"><header><h1>Welcome to the Jungle</h1></header>",
	"<footer>The Bottom</footer></body>"
], _$ssrHydrationKey(), App({}));
const templateEmptied = _$ssr([
	"<html",
	">",
	"",
	"</html>"
], _$ssrHydrationKey(), Head({}), Body({}));
