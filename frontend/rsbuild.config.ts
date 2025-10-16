import { defineConfig } from "@rsbuild/core";
import { pluginReact } from "@rsbuild/plugin-react";
import { pluginSass } from "@rsbuild/plugin-sass";
import { pluginHtmlMinifierTerser } from "rsbuild-plugin-html-minifier-terser";
import { glob } from "glob";

const isProd = process.env.NODE_ENV === "production";
const base = "/_";

const collectLocale = (lang_id: string) => {
  return {
    import: `./src/locale/${lang_id}.ts`,
    filename: `./static/locale/${lang_id}.js`,
    html: false,
    runtime: false,
  };
};
// @ts-ignore
export default defineConfig(async () => {
  const entryPagesFiles = await glob("./src/pages/**/*.ts");
  const entry: Record<string, any> = {};
  entryPagesFiles.forEach((file) => {
    const filename = file.replace("src/", "").replace(".ts", ".js");
    entry[file] = {
      import: `./${file}`,
      html: false,
      runtime: false,
      filename: isProd
        ? `./static/${filename}?v=[contenthash:8]`
        : `./static/${filename}`,
    };
  });
  return {
    plugins: [pluginReact(), pluginSass(), pluginHtmlMinifierTerser()],
    source: {
      entry: {
        index: "./src/index/index.tsx",
        sign_in: "./src/sign_in/index.tsx",
        zh_CN: collectLocale("zh-CN"),
        en_US: collectLocale("en-US"),
        ...entry,
      },
      define: {},
    },
    output: {
      distPath: {
        root: "dist_rsbuild",
      },
      legalComments: "none",
    },
    server: {
      base,
      compress: false,
      proxy: {
        "/api": "http://localhost:8080",
        "/uploads": "http://localhost:8080",
      },
    },
    html: {
      template({ entryName }: { entryName: "index" | "sign_in" }) {
        const templates = {
          index: "./template/index.html",
          sign_in: "./template/sign_in.html",
        };
        return templates[entryName] as string;
      },
      templateParameters: {
        isProd,
      },
    },
    tools: {
      rspack: {
        plugins: [],
        experiments: {
          rspackFuture: { bundlerInfo: { force: false } },
        },
        module: {
          rules: [
            {
              test: /\.ftl$/,
              type: "asset/source", // 表示以纯文本导入
            }
          ]
        },
        externals: {
          "react": "React",
          "react-dom": "ReactDOM",
        },
        optimization: {
          splitChunks: {
            chunks: "async",
          },
        },
      },
    },
  };
});
