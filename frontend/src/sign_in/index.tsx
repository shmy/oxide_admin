import "./index.scss";
import { updateToken } from "../lib/auth";
import xior from "xior";


const redirect =
  new URLSearchParams(window.location.search).get("redirect") ??
  import.meta.env.BASE_URL;

PetiteVue.createApp({
  formData: {
    account: "",
    password: "",
    captcha_key: "",
    captcha_value: "",
  },
  submitting: false,
  signInSucced: false,
  signInError: "",
  captchaError: "",
  captchaRefreshing: false,
  captchaImageUrl: "",
  t(key: string) {
    return _t(key);
  },
  refreshCaptcha() {
    if (this.captchaRefreshing) {
      return;
    }
    this.formData.captcha_key = "";
    this.formData.captcha_value = "";
    this.captchaError = "";
    this.captchaRefreshing = true;
    this.setCaptchaImageUrl("");
    xior
      .get("/api/auth/captcha", {
        responseType: "blob"
      })
      .then((res) => {
        const key = res.headers.get("x-captcha-id");
        if (key) {
          this.formData.captcha_key = key;
          const objectUrl = window.URL.createObjectURL(res.data);
          this.setCaptchaImageUrl(objectUrl);
        } else {
          res.data.text().then((text: string) => {
            try {
              let json = JSON.parse(text);
              this.captchaError = json.msg;
            } catch (_) {
              this.captchaError = text;
            }
          });
        }
      }).catch(e => {
        this.captchaError = e.message;
      }).finally(() => {
        this.captchaRefreshing = false;
      });

  },

  setCaptchaImageUrl(url: string) {
    if (this.captchaImageUrl) {
      window.URL.revokeObjectURL(this.captchaImageUrl);
    }
    this.captchaImageUrl = url;
  },

  handleSubmit() {
    this.signInError = "";
    this.captchaError = "";
    this.submitting = true;
    xior
      .post("/api/auth/sign_in", this.formData)
      .then((res) => {
        if (res.data.status !== 0) {
          throw new Error(res.data.msg);
        }
        this.signInSucced = true;
        const { access_token, refresh_token } = res.data.data;
        updateToken({ access_token, refresh_token });
        window.location.href = redirect;
      })
      .catch((e) => {
        this.signInError = e.message;
        this.refreshCaptcha();
      })
      .finally(() => {
        this.submitting = false;
      });
  }
}).mount('#root');
// const formEl = document.getElementById("form") as HTMLFormElement;
// const submitEl = document.getElementById("submit") as HTMLButtonElement;
// const errorEl = document.getElementById("error") as HTMLDivElement;
// const captchaErrorEl = document.getElementById("captcha-error") as HTMLDivElement;
// const successEl = document.getElementById("success") as HTMLDivElement;
// const captchaWrapperEl = document.getElementById("captcha-wrapper") as HTMLDivElement;
// const captchaKeyEl = document.getElementById("captcha-key") as HTMLInputElement;
// const captchaValueEl = document.getElementById("captcha") as HTMLInputElement;
// const loadingTplEl = document.getElementById("loading-tpl") as HTMLTemplateElement;
// const errorTplEl = document.getElementById("error-tpl") as HTMLTemplateElement;

// const redirect =
//   new URLSearchParams(window.location.search).get("redirect") ??
//   import.meta.env.BASE_URL;

// const setError = (error: string) => {
//   errorEl.innerText = error;
// };

// const setCaptchaError = (error: string) => {
//   captchaErrorEl.innerText = error;
// };

// let captchaRefreshing = false;
// let captchaImageUrl = '';

// const setRefreshing = (refreshing: boolean) => {
//   if (refreshing) {
//     captchaImageUrl = "";
//     captchaKeyEl.value = "";
//     captchaValueEl.value = "";
//     captchaWrapperEl.innerHTML = loadingTplEl.innerHTML;
//   }
//   else if (captchaImageUrl === "") {
//     captchaWrapperEl.innerHTML = errorTplEl.innerHTML;
//   }
//   captchaRefreshing = refreshing;
// };

// const setCaptchaImageUrl = (url: string) => {
//   if (captchaImageUrl) {
//     window.URL.revokeObjectURL(captchaImageUrl);
//   }
//   captchaImageUrl = url;
//   captchaWrapperEl.innerHTML = `<img class="rounded-lg shadow-sm h-full" src="${url}" alt="验证码">`;
// };

// const setSubmitting = (submitting: boolean) => {
//   submitEl.disabled = submitting;
//   if (submitting) {
//     submitEl.innerText = "登录中...";
//   } else {
//     submitEl.innerText = "立即登录";
//   }
// };

// const setSuccess = () => {
//   submitEl.remove();
//   successEl.style.display = "block";
// };

// const refreshCaptcha = () => {
//   if (captchaRefreshing) {
//     return;
//   }

//   setCaptchaError("");
//   setRefreshing(true);
//   xior
//     .get("/api/auth/captcha", {
//       responseType: "blob"
//     })
//     .then((res) => {
//       const key = res.headers.get("x-captcha-id");
//       if (key) {
//         captchaKeyEl.value = key;
//         const objectUrl = window.URL.createObjectURL(res.data);
//         setCaptchaImageUrl(objectUrl);
//       } else {
//         res.data.text().then((text: string) => {
//           try {
//             let json = JSON.parse(text);
//             setCaptchaError(json.msg);
//           } catch (_) {
//             setCaptchaError(text);
//           }
//         });
//       }
//     }).catch(e => {
//       setCaptchaError(e.message);
//     }).finally(() => {
//       setRefreshing(false);
//     });
// };

// formEl?.addEventListener(
//   "submit",
//   (e) => {
//     e.preventDefault();
//     e.stopPropagation();
//     const formData = new FormData(formEl);
//     const account = formData.get("account");
//     const password = formData.get("password");
//     const captcha_key = formData.get("captcha-key");
//     const captcha_value = formData.get("captcha");
//     setError("");
//     setCaptchaError("");
//     setSubmitting(true);
//     xior
//       .post("/api/auth/sign_in", {
//         account,
//         password,
//         captcha_key,
//         captcha_value,
//       })
//       .then((res) => {
//         if (res.data.status !== 0) {
//           throw new Error(res.data.msg);
//         }
//         setSuccess();
//         const { access_token, refresh_token } = res.data.data;
//         updateToken({ access_token, refresh_token });
//         window.location.href = redirect;
//       })
//       .catch((e) => {
//         setError(e.message ?? "提交失败");
//         refreshCaptcha();
//       })
//       .finally(() => {
//         setSubmitting(false);
//       });
//   },
//   false,
// );

// captchaWrapperEl.addEventListener('click', refreshCaptcha, false);

// refreshCaptcha();

// window.addEventListener("beforeunload", () => {
//   if (captchaImageUrl) URL.revokeObjectURL(captchaImageUrl);
// });