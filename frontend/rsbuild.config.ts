import { defineConfig } from "@rsbuild/core";
import { pluginReact } from "@rsbuild/plugin-react";
import { pluginSass } from "@rsbuild/plugin-sass";
import { pluginHtmlMinifierTerser } from "rsbuild-plugin-html-minifier-terser";
import { glob } from "glob";

const isProd = process.env.NODE_ENV === "production";
const base = "/_";

// @ts-ignore
export default defineConfig(async () => {
  const entryPagesFiles = await glob("./src/pages/**/*.ts");
  const entry = {};
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
    plugins: [pluginReact(), pluginSass()],
    source: {
      entry: {
        index: "./src/index/index.ts",
        sign_in: "./src/sign_in/index.tsx",
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
      template({ entryName }) {
        const templates = {
          index: "./template/index.html",
          sign_in: "./template/sign_in.html",
        };
        return templates[entryName];
      },
    },
    tools: {
      rspack: {
        plugins: [],
        experiments: {
          rspackFuture: { bundlerInfo: { force: false } },
        },
        externals: {
          react: "React",
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
