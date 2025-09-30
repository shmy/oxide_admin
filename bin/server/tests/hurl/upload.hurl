# Should success when upload single file given valid file
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

# Should error when upload image file given invalid file
POST {{base_url}}/api/uploads/image
Authorization: Bearer {{access_token}}
[Multipart]
file: file,upload.hurl;
HTTP 200
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" == "Unsupported image format"
jsonpath "$.data" not exists

# Should success when upload image file given valid file
POST {{base_url}}/api/uploads/image
Authorization: Bearer {{access_token}}
[Multipart]
file: file,fixtures/rust-logo.png;
HTTP 200
[Captures]
image: jsonpath "$.data.url"
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data.value" exists
jsonpath "$.data.url" exists

# Should success when fetch image file given previously uploaded image
GET {{base_url}}{{image}}
HTTP 200
[Asserts]
header "Content-Type" contains "image/web"

# Should error when fetch image file given invalid image
GET {{base_url}}/uploads/invalid.webp
HTTP 401

# Should success when start upload chunk given valid filename
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
jsonpath "$.data" exists

# Should success when upload chunk given valid file
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
jsonpath "$.data" exists

# Should success when finish upload chunk given valid parameters
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