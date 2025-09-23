POST {{base_url}}/api/auth/refresh_token
{
    "refresh_token": "{{refresh_token}}"
}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"

POST {{base_url}}/api/profile/password
Authorization: Bearer {{access_token}}
{
    "new_password": "123123",
    "confirm_new_password": "123123",
    "password": "123456"
}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"

POST {{base_url}}/api/profile/sign_out
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
