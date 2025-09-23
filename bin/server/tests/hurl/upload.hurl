POST {{base_url}}/api/uploads/single
Authorization: Bearer {{access_token}}
[Multipart]
file: file,upload.hurl;
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data.value" exists
jsonpath "$.data.url" exists

POST {{base_url}}/api/uploads/image
Authorization: Bearer {{access_token}}
[Multipart]
file: file,upload.hurl;
HTTP 200
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" == "不支持的图片格式"

POST {{base_url}}/api/uploads/image
Authorization: Bearer {{access_token}}
[Multipart]
file: file,fixtures/rust-logo.png;
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data.value" exists
jsonpath "$.data.url" exists

POST {{base_url}}/api/uploads/start_chunk
Authorization: Bearer {{access_token}}
{
    "filename": "rust-logo.png"
}
HTTP 200
[Captures]
key: jsonpath "$.data.key"
upload_id: jsonpath "$.data.uploadId"

[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"

POST {{base_url}}/api/uploads/chunk
Authorization: Bearer {{access_token}}
[Multipart]
file: file,fixtures/rust-logo.png;
key: {{key}}
partNumber: 0
HTTP 200
[Captures]
e_tag: jsonpath "$.data.eTag"

[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"

POST {{base_url}}/api/uploads/finish_chunk
Authorization: Bearer {{access_token}}
{
  "key": "{{key}}",
  "partList": [
    {
      "partNumber": 0
    }
  ],
  "uploadId": "{{upload_id}}"
}
HTTP 200

[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data.value" exists
jsonpath "$.data.url" exists