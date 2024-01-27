# Diagnose and Repair

How to diagnose and repair issues via GraphQL.

Using GraphiQL in the browser will likely time out since the request can
take a very long time. Better to use `curl` as in the examples below.

### Diagnose

```shell
#!/bin/sh
curl -g -X POST -H "Content-Type: application/json" \
     -d '{"query":"query{diagnose(checksum: null) { assetId errorCode } }"}' \
     http://192.168.1.2:3000/graphql
```

### Repair

```shell
#!/bin/sh
curl -g -X POST -H "Content-Type: application/json" \
     -d '{"query":"mutation{repair(checksum: null) { assetId errorCode } }"}' \
     http://192.168.1.2:3000/graphql
```
