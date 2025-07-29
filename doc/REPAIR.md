# Diagnose and Repair

## Database repair

### DuckDB: catalog does not exist

If the app displays an error like `Failure while replaying WAL Catalog ... does not exist` then stop the application and open the database using the `duckdb` client. Hopefully that will process the WAL and sort out whatever was wrong. Then start the application and try loading the home page.

## GraphQL operations

How to diagnose and repair issues via GraphQL.

Using GraphiQL in the browser will likely time out since the request can take a very long time. Better to use `curl` as in the examples below.

### Analyze

```shell
curl -g -X POST -H "Content-Type: application/json" \
     -d '{"query":"query{analyze { totalAssets missingFiles isAnImage hasExifData hasGpsCoords hasOriginalDatetime hasOriginalTimezone } }"}' \
     http://192.168.1.4:3000/graphql
```

### Diagnose

```shell
curl -g -X POST -H "Content-Type: application/json" \
     -d '{"query":"query{diagnose(checksum: null) { assetId errorCode } }"}' \
     http://192.168.1.4:3000/graphql
```

### Repair

```shell
curl -g -X POST -H "Content-Type: application/json" \
     -d '{"query":"mutation{repair(checksum: null) { assetId errorCode } }"}' \
     http://192.168.1.4:3000/graphql
```

### Geocode

It was only necessary to run this one time after introducing the reverse-geocoding feature.

```shell
curl -g -X POST -H "Content-Type: application/json" \
     -d '{"query":"mutation{geocode(overwrite: false)}"}' \
     http://192.168.1.4:3000/graphql
```

### Create dump file

Dump the entire database in [JSON Lines](https://jsonlines.org) text format.

```shell
curl -g -X POST -H "Content-Type: application/json" \
     -d '{"query":"mutation{dump(filepath: \"/assets/dump.json\")}"}' \
     http://192.168.1.4:3000/graphql
```

### Load from dump file

```shell
curl -g -X POST -H "Content-Type: application/json" \
     -d '{"query":"mutation{load(filepath: \"/assets/dump.json\")}"}' \
     http://192.168.1.4:3000/graphql
```

### Finding duplicates

How to find entries in the dump file that have duplicate checksums:

```shell
jq -r '. | "\(.checksum) \(.key)"' assets/dump.json | sort > hashes.txt
cut -d ' ' hashes.txt -f 1 | uniq -d
```
