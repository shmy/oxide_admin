import { buildCrudTable } from "../../lib/table";

export { };

const endpoint = "/system/access_logs";

const schema = {
  type: "page",
  body: buildCrudTable({
    endpoint,
    filters: [

    ],
    headerToolbar: [],
    bulkActions: [

    ],
    operations: [],
    deletable: false,
    showCreatedAt: false,
    showUpdatedAt: false,
    columns: [
      {
        "name": "user_name",
        "label": "用户名称",
      },
      {
        "name": "status",
        "label": "状态码",
      },
      {
        "name": "method",
        "label": "请求方式",
      },
      {
        "name": "uri",
        "label": "请求路径",
      },
      {
        "name": "ip",
        "label": "IP地址",
      },
      {
        "name": "elapsed",
        "label": "请求时长",
        "type": "pretty-ms",
      },
      {
        name: "occurred_at",
        label: "请求时间",
        type: "datetime",
      }
    ],
  }),
};
window._j && window._j(schema);