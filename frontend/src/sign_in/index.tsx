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
  locale: window._locale,
  submitting: false,
  signInSucced: false,
  signInError: "",
  captchaError: "",
  captchaRefreshing: false,
  captchaImageUrl: "",
  t(key: string) {
    return _t(key);
  },
  mounted() {
    this.refreshCaptcha();
  },
  unmonuted() {
    if (this.captchaImageUrl) {
      window.URL.revokeObjectURL(this.captchaImageUrl);
    }
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
  },
  setLanguage(langId: string) {
    xior
      .post("/api/language", {
        lang_id: langId,
      })
      .finally(() => {
        window.location.reload();
      });
  }
}).mount('#root');