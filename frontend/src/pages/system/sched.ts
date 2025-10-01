import { buildCrudTable } from "../../lib/table";

export { };

const endpoint = "/system/scheds";

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
    showCreatedAt: false,
    showUpdatedAt: false,
    columns: [
      {
        name: "key",
        label: "标识",
      },
      {
        name: "name",
        label: "任务名称",
      },
      {
        name: "schedule",
        label: "运行周期",
      },
      {
        name: "succeed",
        label: "是否成功",
        type: "status",
      },
      {
        name: "output",
        label: "运行结果",
      },
      {
        name: "duration_ms",
        label: "运行时长",
      },
      {
        name: "run_at",
        label: "开始时间",
        type: "datetime",
      },
    ],
  }),
};
window._j && window._j(schema);