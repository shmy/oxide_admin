import { FluentBundle, FluentResource } from "@fluent/bundle";
import ftl from "./ftl/zh-CN.ftl";
const bundle = new FluentBundle('zh-CN');
bundle.addResource(new FluentResource(ftl));
window._t = (key, args) => {
    const data = bundle.getMessage(key);
    if (!data) {
        return key;
    }
    return bundle.formatPattern(data!.value!, args);
};