# Diagnose and Repair

How to diagnose and repair issues via GraphQL.

### Diagnose

```shell
#!/bin/sh
curl -g -X POST -H "Content-Type: application/json" \
     -d '{"query":"query{diagnose(checksum: null) { assetId errorCode } }"}' \
     http://192.168.1.1:3000/graphql
```

### Repair

```shell
#!/bin/sh
curl -g -X POST -H "Content-Type: application/json" \
     -d '{"query":"mutation{repair(checksum: null) { assetId errorCode } }"}' \
     http://192.168.1.1:3000/graphql
```
