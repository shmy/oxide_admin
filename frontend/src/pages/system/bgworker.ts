export { };

const endpoint = "/system/bgworkers";

const buildJobList = (status: string) => {
  return {
    type: "crud",
    id: status.toLowerCase(),
    api: `${endpoint}/$name/jobs/${status}/$page`,
    columns: [
      {
        label: "参数",
        value: "${args | json}"
      },
      {
        label: "状态",
        name: "parts.context.status"
      },
      {
        label: "输出",
        name: "parts.context.last_error"
      },
      {
        label: "开始时间",
        type: "datetime",
        name: "parts.context.run_at"
      },
      {
        label: "结束时间",
        type: "datetime",
        name: "parts.context.done_at"
      },
    ]
  };
};

const builderWorkerDrawer = () => {
  return {
    title: "$name",
    size: "xl",
    body: {
      type: "service",
      id: "tabs",
      api: `${endpoint}/$name/stat`,
      body: {
        type: "tabs",
        tabsMode: "tiled",
        toolbar: [
          {
            type: "button",
            icon: "fas fa-refresh",
            actionType: "reload",
            target: "tabs,pending,running,done,failed,killed"
          }
        ],
        tabs: [
          {
            title: "队列中($pending)",
            reload: true,
            body: buildJobList("Pending")
          },
          {
            title: "执行中($running)",
            reload: true,
            body: buildJobList("Running")
          },
          {
            title: "已成功($done)",
            reload: true,
            body: buildJobList("Done")
          },
          {
            title: "已失败($failed)",
            reload: true,
            body: buildJobList("Failed")
          },
          {
            title: "已放弃($killed)",
            reload: true,
            body: buildJobList("Killed")
          }
        ]
      }
    },
    actions: []
  }
};

const schema = {
  type: "page",
  body: {
    type: "service",
    api: endpoint,
    interval: 3000,
    silentPolling: true,
    body: {
      type: "list",
      source: "$items",
      listItem: {
        title: "${name}",
        subTitle: "并发: ${concurrency} | 重试: ${retries} | 超时: ${timeout|duration}",
        desc: "队列中 ${pending} | 执行中 ${running} | 已成功 ${done} | 已失败 ${failed} | 已放弃 ${killed}",
      },
      itemAction: {
        type: "button",
        actionType: "drawer",
        drawer: builderWorkerDrawer()
      },
    }
  },
};
window._j && window._j(schema);