import { buildCrudTable } from "../../lib/table";

export { };

const endpoint = "/organization/departments";

const buildDrawer = (isAdd = true) => {
  const label = isAdd ? "Create department" : null;
  const title = isAdd ? "Create department" : "Edit department";
  const level = isAdd ? "primary" : "link";
  const icon = isAdd ? "fas fa-plus" : "fas fa-edit";
  const tooltip = isAdd ? null : "Edit";

  const api = isAdd ? `post:${endpoint}` : `put:${endpoint}/$id`;
  const initApi = isAdd ? null : `get:${endpoint}/$id`;

  return {
    label: label,
    icon: icon,
    tooltip: tooltip,
    type: "button",
    align: "right",
    actionType: "drawer",
    level: level,
    drawer: {
      title: title,
      size: "md",
      body: {
        type: "form",
        canAccessSuperData: false,
        api: api,
        initApi: initApi,
        body: [

        ],
      },
    },
  };
};

const schema = {
  type: "page",
  body: buildCrudTable({
    endpoint,
    filters: [

    ],
    headerToolbar: [buildDrawer()],
    bulkActions: [

    ],
    operations: [buildDrawer(false)],
    columns: [

    ],
  }),
};
window._j && window._j(schema);