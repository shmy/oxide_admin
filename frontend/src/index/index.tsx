import "./index.scss";
import {
  history,
  isCurrentUrl,
  jumpTo,
  updateLocation,
} from "../lib/amis_router";
import { getAccessToken, redirectToSignIn, signOut } from "../lib/authn";
import http from "../lib/http";
import { logoUrl } from "../lib/constant";
import { registerComponents } from "../lib/component";

if (!getAccessToken()) {
  redirectToSignIn();
}

const amisLib = amisRequire("amis");
const amis = amisRequire("amis/embed");
registerComponents(amisLib);

const buildDropdown = () => {
  return {
    type: "dropdown-button",
    style: {
      "--portrait-url": `url('\${user.portrait}'), url('${logoUrl}')`,
    },
    className: "header-dropdown",
    label: "${user.name}(${user.account})",
    size: "lg",
    trigger: "click",
    buttons: [
      {
        type: "button",
        label: " 修改密码",
        icon: "fas fa-key",
        actionType: "link",
        link: "/profile/update_password",
      },
      {
        type: "button",
        label: " 退出登录",
        icon: "fas fa-sign-out",
        actionType: "dialog",
        dialog: {
          title: "注销",
          body: "确定要注销登录吗？",
          actions: [
            {
              type: "button",
              label: "取消",
              close: true,
            },
            {
              type: "button",
              level: "primary",
              label: "确定",
              onClick: signOut,
            },
          ],
        },
      },
    ],
  };
};

const assetPrefix = import.meta.env.ASSET_PREFIX;

const schemas = {
  type: "app",
  brandName: "Oxide Admin",
  logo: assetPrefix + "/static/image/logo.png",
  api: {
    method: "get",
    url: "/profile/current",
    adaptor: (response: any) => {
      window._permissions = new Set(response.data.permissions);
      window._hasPermission = (permission: number) => window._permissions.has(permission);
      return response;
    }
  },
  asideBefore: {
    type: "container",
    className: "text-center my-2 hidden m:block",
    body: [buildDropdown()],
  },
  header: {
    type: "container",
    className: "w-full ",
    bodyClassName: "w-full flex justify-end items-center gap-4 px-4",
    body: [buildDropdown()],
  },
};

const amisScoped = amis.embed(
  "#root",
  schemas,
  {
    locale: "zh-CN",
    context: {},
  },
  {
    updateLocation,
    jumpTo,
    isCurrentUrl,
    theme: "antd",
    fetcher: (config: any) => {
      let headers: Record<string, string> = {
        "Content-Type": "application/json",
      };
      if (config.data instanceof FormData) {
        delete headers["Content-Type"];
      }
      const controller = new AbortController();
      config.config?.cancelExecutor?.(() => {
        controller.abort();
      });
      return http.request({
        url: config.url,
        method: config.method,
        data: config.data,
        headers,
        responseType: config.responseType,
        signal: controller.signal,
        onUploadProgress: config.config.onUploadProgress,
        onDownloadProgress: config.config.onDownloadProgress,
      });
    },
  },
);

history.listen((state: any) => {
  amisScoped.updateProps({
    location: state.location || state,
  });
});
