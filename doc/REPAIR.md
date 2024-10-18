# Diagnose and Repair

How to diagnose and repair issues via GraphQL.

Using GraphiQL in the browser will likely time out since the request can
take a very long time. Better to use `curl` as in the examples below.

### Analyze

```shell
curl -g -X POST -H "Content-Type: application/json" \
     -d '{"query":"query{analyze { totalAssets missingFiles isAnImage hasExifData hasGpsCoords hasOriginalDatetime hasOriginalTimezone } }"}' \
     http://192.168.1.2:3000/graphql
```

### Diagnose

```shell
curl -g -X POST -H "Content-Type: application/json" \
     -d '{"query":"query{diagnose(checksum: null) { assetId errorCode } }"}' \
     http://192.168.1.2:3000/graphql
```

### Repair

```shell
curl -g -X POST -H "Content-Type: application/json" \
     -d '{"query":"mutation{repair(checksum: null) { assetId errorCode } }"}' \
     http://192.168.1.2:3000/graphql
```

### Geocode

```shell
curl -g -X POST -H "Content-Type: application/json" \
     -d '{"query":"mutation{geocode(overwrite: false)}"}' \
     http://192.168.1.2:3000/graphql
```
