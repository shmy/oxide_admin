export { };
const endpoint = "/organization/departments";

const buildDepartment = () => {

  return {
    "type": "input-tree",
    "name": "department",
    "source": endpoint,
    "heightAuto": true,
    "creatable": true,
    "removable": true,
    "editable": true,
    "required": true,
    "showOutline": true,
    "addApi": `post:${endpoint}`,
    "editApi": `put:${endpoint}/\${id}`,
    "deleteApi": {
      url: `${endpoint}/batch/delete`,
      method: "POST",
      data: {
        "ids": ["\${id}"]
      }
    },
    "options": [],
    "menuTpl": "${label} [${value}]",
    "addControls": [
      {
        "label": "名称",
        "type": "input-text",
        "required": true,
        "name": "name"
      },
      {
        "label": "编号",
        "type": "input-text",
        "required": true,
        "name": "code"
      },
    ],
    "editControls": [
      {
        "label": "ID",
        "type": "hidden",
        "required": true,
        "name": "id"
      },
      {
        "label": "名称",
        "type": "input-text",
        "required": true,
        "name": "name",
        "value": "${label}"
      },
    ],
  };
};

const schema = {
  "type": "page",
  "body": {
    "type": "panel",
    "body": [
      buildDepartment()
    ]
  }
};
window._j && window._j(schema);