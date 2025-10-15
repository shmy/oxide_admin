import { FluentBundle, FluentResource } from "@fluent/bundle";
import ftl from "./ftl/en_US.ftl";
const bundle = new FluentBundle('en_US');
bundle.addResource(new FluentResource(ftl));
window._t = (key, args) => {
    const data = bundle.getMessage(key);
    if (!data) {
        return key;
    }
    return bundle.formatPattern(data!.value!, args);
};