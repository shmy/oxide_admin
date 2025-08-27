export {};

const schema = {
  type: "page",
  body: {
    type: "form",
    title: "修改密码",
    api: "put:/profile/password",
    body: [
      {
        label: "旧密码",
        name: "password",
        required: true,
        type: "input-password",
      },
      {
        label: "新密码",
        name: "new_password",
        required: true,
        type: "input-password",
      },
      {
        label: "确认密码",
        name: "confirm_new_password",
        required: true,
        type: "input-password",
      },
    ],
  },
};
window._j && window._j(schema);
