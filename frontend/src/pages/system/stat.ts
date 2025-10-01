export { };

const schema = {
  type: "page",
  body: [
    {
      type: "form",
      title: "",
      initApi: "/system/stats",
      static: true,
      body: [
        {
          type: "group",
          body: [
            {
              type: "input-text",
              label: "系统名称",
              name: "system.os_name",
            },
            {
              type: "input-text",
              label: "系统版本",
              name: "system.long_os_version",
            },
            {
              type: "input-text",
              label: "主机名称",
              name: "system.host_name",
            },

            {
              type: "input-text",
              label: "CPU架构",
              name: "system.cpu_arch",
            },
          ],
        },
        {
          type: "group",
          body: [
            {
              type: "input-text",
              label: "CPU物理核心数",
              name: "system.physical_core_count",
            },
            {
              type: "static",
              label: "CPU逻辑核心数",
              value: "${system.cpus.length}",
            },
            {
              type: "pretty-bytes",
              label: "系统总内存",
              name: "system.total_memory",
            },
            {
              type: "pretty-bytes",
              label: "系统总交换内存",
              name: "system.total_swap",
            },
          ],
        },
        {
          type: "group",
          body: [
            {
              type: "input-datetime",
              label: "系统启动时间",
              name: "system.boot_time",
            },
            {
              type: "input-text",
              label: "进程pid",
              name: "process.pid",
            },
            {
              type: "input-text",
              label: "进程名称",
              name: "process.name",
            },
            {
              type: "input-text",
              label: "进程路径",
              name: "process.exe",
            },
          ],
        },
        {
          type: "group",
          body: [
            {
              type: "input-text",
              label: "进程工作路径",
              name: "process.cwd",
            },
            {
              type: "input-datetime",
              label: "进程启动时间",
              name: "process.start_time",
            },
            {
              type: "input-text",
            },
            {
              type: "input-text",
            },
          ],
        },
      ],
      actions: [],
    },
  ],
};
window._j && window._j(schema);
