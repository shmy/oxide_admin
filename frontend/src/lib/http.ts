import xior, { type XiorRequestConfig } from "xior";
import uploadDownloadProgressPlugin from "xior/plugins/progress";

import { getAccessToken, redirectToSignIn, refreshToken } from "./authn";
import { attachmentAdpator } from "./attachmentAdpator";

const http = xior.create({
  baseURL: "/api",
  headers: {
    // 'Content-Type': 'application/json'
  },
});

http.plugins.use(
  uploadDownloadProgressPlugin({
    progressDuration: 5 * 1000,
  }),
);

http.interceptors.request.use(
  (config) => {
    let access_token = getAccessToken();
    if (access_token) {
      config.headers["Authorization"] = "Bearer " + access_token;
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  },
);

http.interceptors.response.use(
  (response) => {
    return attachmentAdpator(response, () => "");
  },
  (error) => {
    if (error.response?.status === 401) {
      return refreshToken(http).then((succeed) => {
        if (succeed) {
          return http.request(error.config as XiorRequestConfig);
        }
        redirectToSignIn();
        return Promise.reject(error);
      });
    }
    return Promise.reject({ message: `操作失败：${error.message}` });
  },
);

export default http;
