export {};

const schema = {
  type: "page",
  body: {
    title: "",
    type: "form",
    api: "post:/profile/upgrade",
    body: [
      {
        type: "react-test",
      },
      {
        type: "alert",
        body: "请注意：更新过程中会导致短暂停止服务。",
        level: "warning",
        showIcon: true,
      },
      {
        type: "input-file",
        name: "path",
        label: "程序文件",
        startChunkApi: "/uploads/start_chunk",
        chunkApi: "/uploads/chunk",
        // accept: ".zst",
        finishChunkApi: "/uploads/finish_chunk",
        maxSize: 200 * 1024 * 1024,
        chunkSize: 2 * 1024 * 1024,
        useChunk: true,
        maxLength: 1,
        required: true,
      },
      {
        type: "input-file",
        name: "image",
        label: "图片",
        receiver: "/uploads/image",
        maxLength: 1,
        required: true,
      },
      {
        type: "input-file",
        name: "single",
        label: "单文件",
        receiver: "/uploads/single",
        maxLength: 1,
        required: true,
      },
      {
        type: "input-password",
        name: "password",
        label: "登录密码",
        description: "该操作属于危险操作，请提供登录密码以验证。",
        required: true,
      },
    ],
    feedback: {
      title: "更新完毕",
      body: "请点击刷新页面按钮，如果刷新失败，请手动重启",
      showCloseButton: false,
      actions: [
        {
          type: "button",
          level: "primary",
          label: "刷新页面",
          onClick: "window.location.reload();",
        },
      ],
    },
  },
};
window._j && window._j(schema);
