import { defineRouteMiddleware } from "@astrojs/starlight/route-data";
import { isWideTablePage } from "./lib/wide-table-pages";

export const onRequest = defineRouteMiddleware((context) => {
  const route = context.locals.starlightRoute;
  // Give wide reference tables the space of the "On this page" pane.
  if (isWideTablePage(route.id)) {
    route.toc = undefined;
  }
});
