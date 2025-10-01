import { getAccessToken } from "../../lib/authn";
import { usedStatuses } from "../../lib/options";
import { buildCrudTable } from "../../lib/table";

export { };

const endpoint = "/system/files";

const schema = {
  type: "page",
  body: buildCrudTable({
    endpoint,
    deletable: false,
    filters: [
      {
        type: "input-text",
        name: "name",
        label: "文件名称",
        placeholder: "请输入文件名称",
        clearable: true,
      },
      {
        type: "select",
        name: "used",
        label: "是否使用",
        placeholder: "请选择是否使用",
        clearable: true,
        options: usedStatuses,
      },
    ],
    bulkActions: [

    ],
    showUpdatedAt: false,
    operations: [
      {
        icon: "fas fa-download",
        actionType: "url",
        type: "button",
        align: "right",
        tooltip: "下载",
        level: "link",
        url: "/api/system/files/download/$path?access_token=" + getAccessToken(),
        blank: true,
      }
    ],
    columns: [
      {
        name: "name",
        label: "文件名称",
      },
      {
        type: "pretty-bytes",
        label: "文件大小",
        name: "size",
      },
      {
        name: "used",
        label: "使用中",
        type: "status",
      },
      {
        name: "path",
        label: "文件路径",
      },
    ],
  }),
};
window._j && window._j(schema);