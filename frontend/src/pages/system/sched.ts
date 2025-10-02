import { ifElementAuthorized } from "../../lib/authn";
import { PERMISSIONS } from "../../lib/permissions";
import { buildCrudTable } from "../../lib/table";

export { };

const endpoint = "/system/scheds";

const buildRecord = () => {
  return {
    icon: "fas fa-eye",
    align: "right",
    tooltip: "执行记录",
    level: "link",
    actionType: "drawer",
    drawer: {
      title: "$name",
      size: "xl",
      body: buildCrudTable({
        endpoint: `${endpoint}/records?key=$key&page=$page&page_size=$page_size`,
        deleteEndpoint: `${endpoint}/records`,
        columns: [
          {
            name: "key",
            label: "任务标识",
          },
          {
            name: "name",
            label: "任务名称",
          },
          {
            name: "expr",
            label: "运行周期",
          },
          {
            name: "succeed",
            label: "运行状态",
            type: "status",
          },
          {
            name: "result",
            label: "运行结果",
          },
          {
            name: "run_at",
            label: "运行时间",
            type: "datetime",
          },
          {
            name: "duration_ms",
            label: "运行时长",
            type: "pretty-ms",
          },
        ]
      }),
      actions: []
    },
  };
};

const schema = {
  type: "page",
  body: buildCrudTable({
    endpoint,
    filters: [

    ],
    headerToolbar: [],
    bulkActions: [

    ],
    operations: [
      ...ifElementAuthorized(PERMISSIONS.SYSTEM.SCHED.READ, buildRecord()),

    ],
    showCreatedAt: false,
    showUpdatedAt: false,
    deletable: false,
    columns: [
      {
        name: "key",
        label: "任务标识",
      },
      {
        name: "name",
        label: "任务名称",
      },
      {
        name: "expr",
        label: "运行周期",
      },
      {
        name: "last_succeed",
        label: "上次运行状态",
        type: "status",
      },
      {
        name: "last_result",
        label: "上次运行结果",
      },
      {
        name: "last_run_at",
        label: "上次运行时间",
        type: "datetime",
      },
      {
        name: "last_duration_ms",
        label: "上次运行时长",
        type: "pretty-ms",
      },
      {
        name: "next_run_at",
        label: "下次运行时间",
        type: "datetime",
      },
    ],
  }),
};
window._j && window._j(schema);