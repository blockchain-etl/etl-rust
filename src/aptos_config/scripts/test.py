import http.client
import json

headers = { 'Accept': "application/json" }

conn = http.client.HTTPSConnection("fullnode.devnet.aptoslabs.com")


conn.request("GET", "/v1/", headers=headers)

res = conn.getresponse()
data = res.read()

data_json = json.loads(data.decode("utf-8"))
print(data_json)

print(f"Current block height: {data_json['block_height']}")


conn.request("GET", f"/v1/blocks/by_height/906107?with_transactions=true", headers=headers)

res = conn.getresponse()
data = res.read()

print(data.decode("utf-8"))