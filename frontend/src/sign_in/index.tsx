import { updateToken } from "../lib/authn";
import http from "../lib/http";
import "./index.scss";

const formEl = document.getElementById("form") as HTMLFormElement;
const submitEl = document.getElementById("submit") as HTMLButtonElement;
const errorEl = document.getElementById("error") as HTMLDivElement;
const successEl = document.getElementById("success") as HTMLDivElement;
const redirect =
  new URLSearchParams(window.location.search).get("redirect") ??
  import.meta.env.BASE_URL;

const setError = (error: string) => {
  errorEl.innerText = error;
};

const setSubmitting = (submitting: boolean) => {
  submitEl.disabled = submitting;
  if (submitting) {
    submitEl.innerText = "登录中...";
  } else {
    submitEl.innerText = "立即登录";
  }
};

const setSuccess = () => {
  submitEl.remove();
  successEl.style.display = "block";
};
formEl?.addEventListener(
  "submit",
  (e) => {
    e.preventDefault();
    e.stopPropagation();
    const formData = new FormData(formEl);
    const account = formData.get("account");
    const password = formData.get("password");
    setError("");
    setSubmitting(true);
    http
      .post("/auth/sign_in", {
        account,
        password,
      })
      .then((res) => {
        if (res.data.status !== 0) {
          setError(res.data.msg);
          return;
        }
        setSuccess();
        const { access_token, refresh_token } = res.data.data;
        updateToken({ access_token, refresh_token });
        window.location.href = redirect;
      })
      .catch((e) => {
        setError(e.message ?? "提交失败");
      })
      .finally(() => {
        setSubmitting(false);
      });
  },
  false,
);
