export { };

const endpoint = "/system/caches";

const schema = {
  type: "page",
  body: {
    "type": "input-tree",
    "name": "caches",
    "source": endpoint,
    "heightAuto": true,
    "showOutline": true,
    "onlyLeaf": true,
    "deleteApi": {
      url: `${endpoint}/delete`,
      method: "POST",
      data: {
        prefix: "\$value"
      }
    },
    "onEvent": {
      "change": {
        "actions": [
          {
            actionType: "custom",
            script: (_p1: any, _p2: any, props: any) => {
              window.amisScoped.doAction({
                actionType: "drawer",
                args: {
                  drawer: {
                    title: props.data.value,
                    size: "lg",
                    body: [
                      {
                        type: "service",
                        api: `${endpoint}/${props.data.value}`,
                        body: [
                          {
                            type: "datetime",
                            name: "expired_at",
                            displayFormat: "过期时间：YYYY-MM-DD HH:mm:ss",
                          },
                          {
                            type: "code",
                            name: "value",
                            language: "json",
                          },
                        ]
                      }
                    ],
                    actions: []
                  },
                }
              });
            }
          }
        ]
      }
    },
    "options": [],
    "menuTpl": "${label}",
  },
};
window._j && window._j(schema);