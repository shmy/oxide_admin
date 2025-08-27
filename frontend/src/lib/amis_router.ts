// @ts-ignore
import { createBrowserHistory } from "history";

const match = amisRequire("path-to-regexp").match;
const base = import.meta.env.BASE_URL;

export const history = createBrowserHistory({
  basename: base,
});
const normalizeLink = (to: string, location = history.location) => {
  to = to || "";

  if (to && to[0] === "#") {
    to = location.pathname + location.search + to;
  } else if (to && to[0] === "?") {
    to = location.pathname + to;
  }

  const idx = to.indexOf("?");
  const idx2 = to.indexOf("#");
  let pathname = ~idx
    ? to.substring(0, idx)
    : ~idx2
      ? to.substring(0, idx2)
      : to;
  const search = ~idx ? to.substring(idx, ~idx2 ? idx2 : undefined) : "";
  const hash = ~idx2 ? to.substring(idx2) : location.hash;

  if (!pathname) {
    pathname = location.pathname;
  } else if (pathname[0] !== "/" && !/^https?:\/\//.test(pathname)) {
    const relativeBase = location.pathname;
    const paths = relativeBase.split("/");
    paths.pop();
    let m: RegExpExecArray | null;
    while (true) {
      m = /^\.\.?\//.exec(pathname);
      if (!m) {
        break;
      }
      if (m[0] === "../") {
        paths.pop();
      }
      pathname = pathname.substring(m[0].length);
    }
    pathname = paths.concat(pathname).join("/");
  }

  return pathname + search + hash;
};

export const isCurrentUrl = (to: string, ctx?: any) => {
  if (!to) {
    return false;
  }
  const pathname = history.location.pathname;
  const link = normalizeLink(to, {
    ...location,
    pathname,
    hash: "",
  });

  if (!~link.indexOf("http") && ~link.indexOf(":")) {
    const strict = ctx?.strictct;
    return match(link, {
      decode: decodeURIComponent,
      strict: typeof strict !== "undefined" ? strict : true,
    })(pathname);
  }

  return decodeURI(pathname) === link;
};

export const updateLocation = (location: string, replace: boolean) => {
  location = normalizeLink(location);
  if (location === "goBack") {
    return history.goBack();
  } else if (
    (!/^https?:\/\//.test(location) &&
      location === history.location.pathname + history.location.search) ||
    location === history.location.href
  ) {
    // 目标地址和当前地址一样，不处理，免得重复刷新
    return;
  } else if (/^https?:\/\//.test(location) || !history) {
    window.location.href = location;
    return;
  }

  history[replace ? "replace" : "push"](location);
};

export const jumpTo = (to: string, action: any) => {
  if (to === "goBack") {
    return history.goBack();
  }

  to = normalizeLink(to);

  if (isCurrentUrl(to)) {
    return;
  }

  if (action && action.actionType === "url") {
    if (action.blank === false) {
      window.location.href = to;
    } else {
      window.open(to, "_blank");
    }
    return;
  } else if (action?.blank) {
    window.open(to, "_blank");
    return;
  }

  if (/^https?:\/\//.test(to)) {
    window.location.href = to;
  } else if (
    (!/^https?:\/\//.test(to) &&
      to === history.pathname + history.location.search) ||
    to === history.location.href
  ) {
    // do nothing
  } else {
    history.push(to);
  }
};
