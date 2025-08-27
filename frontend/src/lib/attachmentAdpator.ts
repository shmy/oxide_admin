// @ts-ignore
import { saveAs } from "file-saver";

/**
 * 处理接口返回附件的情况。
 * 支持提取 filename*=UTF-8'' 以及普通 filename=，并修正编码问题。
 */
export async function attachmentAdpator(
  response: any,
  __: Function,
  api?: any,
) {
  const disposition = response?.headers?.get("content-disposition") || "";
  const contentType = response?.headers?.get("content-type") || "";

  // 1. 检查是否是附件
  if (disposition.includes("attachment")) {
    let filename = "";

    // 1.1 如果 API 上配置了下载文件名，优先使用
    if (api?.downloadFileName) {
      filename = api.downloadFileName;
    } else {
      // 1.2 提取 disposition 中的文件名
      const filenameStarMatch = disposition.match(/filename\*\s*=\s*([^;]+)/i);
      if (filenameStarMatch) {
        // 处理 filename*=UTF-8''xxx
        let encodedFilename = filenameStarMatch[1];
        const utf8Match = encodedFilename.match(/UTF-8''(.+)/i);
        if (utf8Match) {
          filename = decodeURIComponent(utf8Match[1]);
        } else {
          filename = decodeURIComponent(encodedFilename);
        }
      } else {
        // fallback 提取普通 filename=
        const filenameMatch = disposition.match(
          /filename\s*=\s*(['"]?)([^'";]+)\1/i,
        );
        if (filenameMatch) {
          filename = filenameMatch[2];
        }
      }

      // 修复可能错误转义的 +
      filename = filename.replace(/\+/g, " ").trim();

      // 防止路径穿越
      filename = filename.split("/").pop()?.split("\\").pop() || filename;
    }
    // 2. 创建 Blob
    const blob =
      response.data instanceof Blob
        ? response.data
        : new Blob([response.data], { type: contentType });

    saveAs(blob, filename);

    return {
      ...response,
      data: {
        status: 0,
        msg: __("Embed.downloading"),
      },
    };
  }

  // 3. 如果是 Blob，但实际上是 JSON 错误（比如 token 失效），需要读出内容
  if (response.data && response.data instanceof Blob) {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onloadend = () => {
        try {
          const text = reader.result as string;
          const parsed = JSON.parse(text);
          resolve({
            ...response,
            data: parsed,
          });
        } catch (err) {
          reject(err);
        }
      };
      reader.onerror = reject;
      reader.readAsText(response.data);
    });
  }

  // 4. 普通返回
  return response;
}

export default attachmentAdpator;
