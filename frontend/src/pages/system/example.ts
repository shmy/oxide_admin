export {};

const schema = {
  type: "page",
  body: [
    {
      type: "react-test",
    },
    {
      type: "input-file",
      name: "path",
      label: "分片上传",
      startChunkApi: "/uploads/start_chunk",
      chunkApi: "/uploads/chunk",
      finishChunkApi: "/uploads/finish_chunk",
      maxSize: 200 * 1024 * 1024,
      chunkSize: 2 * 1024 * 1024,
      useChunk: true,
      maxLength: 1,
      required: true,
    },
    {
      type: "input-file",
      name: "image",
      label: "图片上传",
      accept: "image/*",
      maxSize: 2 * 1024 * 1024,
      receiver: "/uploads/image",
      maxLength: 1,
      required: true,
    },
    {
      type: "input-file",
      name: "single",
      label: "单文件上传",
      maxSize: 2 * 1024 * 1024,
      receiver: "/uploads/single",
      maxLength: 1,
      required: true,
    },
  ],
};
window._j && window._j(schema);
