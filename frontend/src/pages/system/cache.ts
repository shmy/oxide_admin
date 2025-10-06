export { };

const endpoint = "/system/caches";

const schema = {
  type: "page",
  body: {
    "type": "input-tree",
    "name": "caches",
    "source": endpoint,
    "heightAuto": true,
    "required": true,
    "deleteApi": {
      url: `${endpoint}/delete`,
      method: "POST",
      data: {
        "prefix": "\$value"
      }
    },
    "options": [],
    "menuTpl": "${label}",
  },
};
window._j && window._j(schema);