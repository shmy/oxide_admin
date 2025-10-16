import "./index.scss";
import {
  history,
  isCurrentUrl,
  jumpTo,
  updateLocation,
} from "../lib/amis_router";
import { getAccessToken, redirectToSignIn, signOut } from "../lib/auth";
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
        label: " " + _t('switch_language'),
        icon: "fas fa-language",
        actionType: "dialog",
        dialog: {
          title: _t('switch_language'),
          body: {
            type: "form",
            initApi: "/profile/language",
            api: "/language",
            body: [
              {
                type: "select",
                label: _t('language'),
                name: "lang_id",
                options: [
                  { value: "en-US", label: "English" },
                  { value: "zh-CN", label: "简体中文" },
                ],
              }
            ],
            onEvent: {
              submitSucc: {
                actions: [
                  {
                    actionType: "custom",
                    script: "window.location.reload()"
                  }
                ]
              }
            }
          }
        },
      },
      {
        type: "button",
        label: " " + _t('change_password'),
        icon: "fas fa-key",
        actionType: "link",
        link: "/profile/update_password",
      },
      {
        type: "button",
        label: " " + _t('sign_out'),
        icon: "fas fa-sign-out",
        actionType: "dialog",
        dialog: {
          title: _t('sign_out'),
          body: _t('are_you_sure_to_sign_out'),
          actions: [
            {
              type: "button",
              label: _t('cancel'),
              close: true,
            },
            {
              type: "button",
              level: "primary",
              label: _t('confirm'),
              onClick: signOut,
            },
          ],
        },
      },
    ],
  };
};


const schemas = {
  type: "app",
  brandName: "Oxide Admin",
  logo: logoUrl,
  api: {
    method: "get",
    url: "/profile",
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
    locale: _locale,
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

window.amisScoped = amisScoped;
