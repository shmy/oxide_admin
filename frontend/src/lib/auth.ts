import { type XiorInstance } from "xior";

let accessTokenKey = "access_token";
let refreshTokenKey = "refresh_token";
let latestAccessToken = window.localStorage.getItem(accessTokenKey) || "";
let latestRefreshToken = window.localStorage.getItem(refreshTokenKey) || "";
let refreshTokenPromise: Promise<boolean> | null = null;
const base = import.meta.env.BASE_URL;

window.addEventListener("storage", (event) => {
  if (event.key === accessTokenKey) {
    latestAccessToken = event.newValue || "";
  }
  if (event.key === refreshTokenKey) {
    latestRefreshToken = event.newValue || "";
  }
});

export const refreshToken = (http: XiorInstance) => {
  if (!latestRefreshToken) {
    return Promise.resolve(false);
  }
  if (!refreshTokenPromise) {
    refreshTokenPromise = http
      .post<{ data: TokenData }>("/auth/token", {
        token: latestRefreshToken,
      })
      .then((response) => {
        updateToken(response.data.data);
        return true;
      })
      .catch((_) => {
        return false;
      })
      .finally(() => {
        refreshTokenPromise = null;
      });
  }
  return refreshTokenPromise;
};

export const signOut = () => {
  if (window.navigator.sendBeacon) {
    const access_token = getAccessToken();
    window.navigator.sendBeacon(
      `/api/profile/sign_out?access_token=${access_token}`,
    );
  }
  window.localStorage.clear();
  redirectToSignIn();
};

export const redirectToSignIn = () => {
  const pathname = window.location.pathname;
  window.location.href = `${base}/sign_in?redirect=${pathname}`;
};

export const getAccessToken = () => {
  return latestAccessToken;
};

type TokenData = {
  access_token: string;
  refresh_token: string;
};

export const updateToken = (data: TokenData) => {
  latestAccessToken = data.access_token;
  latestRefreshToken = data.refresh_token;
  window.localStorage.setItem(accessTokenKey, latestAccessToken);
  window.localStorage.setItem(refreshTokenKey, latestRefreshToken);
};

type Elem<T> = T | T[];

export const ifElementAuthorized = <T>(permission: number, elem: Elem<T>): T[] =>
  _hasPermission(permission) ? ([] as T[]).concat(elem) : [];