export { };

const schema = {
  type: "page",
  body: {
    type: "form",
    title: "",
    api: "put:/profile/password",
    body: [
      {
        label: _t('current_password'),
        name: "password",
        required: true,
        type: "input-password",
      },
      {
        label: _t('new_password'),
        name: "new_password",
        required: true,
        type: "input-password",
      },
      {
        label: _t('confirm_new_password'),
        name: "confirm_new_password",
        required: true,
        type: "input-password",
      },
    ],
  },
};
window._j && window._j(schema);
