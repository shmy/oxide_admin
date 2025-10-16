import { FluentBundle, FluentResource } from "@fluent/bundle";
import ftl from "./ftl/en-US.ftl";
const bundle = new FluentBundle('en-US');
bundle.addResource(new FluentResource(ftl));
window._t = (key, args) => {
    const data = bundle.getMessage(key);
    if (!data) {
        return key;
    }
    return bundle.formatPattern(data!.value!, args);
};