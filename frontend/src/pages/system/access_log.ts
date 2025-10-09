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
    columns: [
      {
        "name": "user_id",
        "label": "用户ID",
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
        "name": "ip_region",
        "label": "IP地区",
      },
      {
        "name": "status",
        "label": "状态码",
      },
      {
        "name": "elapsed",
        "label": "请求时长",
        "type": "pretty-ms",
      },
      {
        "name": "user_agent",
        "label": "客户端",
      },
    ],
  }),
};
window._j && window._j(schema);