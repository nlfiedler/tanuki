# Diagnose and Repair

## Database dump/load

How to diagnose and repair issues via GraphQL. Using GraphiQL in the browser will likely time out since the request can take a very long time. Better to use `curl` as in the examples below.

### Create dump file

Dump the entire database in [JSON Lines](https://jsonlines.org) text format.

```shell
curl -o dump.json http://192.168.1.4:3000/records/dump
```

### Load from dump file

```shell
curl -F dump=@dump.json http://192.168.1.4:3000/records/load
```

### Fetching Tags

```shell
curl -g -X POST -H "Content-Type: application/json" \
     -d '{"query":"query { tags { label count } }"}' \
     http://192.168.1.4:3000/graphql > tags.json

jq -r '.data.tags | sort_by(.label) | .[] | "\(.label),\(.count)"' tags.json > sorted-tags.csv
```

### Fetching Media Types

```shell
curl -g -X POST -H "Content-Type: application/json" \
     -d '{"query":"query { mediaTypes { label count } }"}' \
     http://192.168.1.4:3000/graphql > mediaTypes.json

jq -r '.data.mediaTypes | sort_by(.label) | .[] | "\(.label),\(.count)"' mediaTypes.json > sorted-mediaTypes.csv
```

### Fetching Years

```shell
curl -g -X POST -H "Content-Type: application/json" \
     -d '{"query":"query { years { label count } }"}' \
     http://192.168.1.4:3000/graphql > years.json

jq -r '.data.years | sort_by(.label) | .[] | "\(.label),\(.count)"' years.json > sorted-years.csv
```

### Fetching Locations

```shell
curl -g -X POST -H "Content-Type: application/json" \
     -d '{"query":"query { locationRecords { label city region } }"}' \
     http://192.168.1.4:3000/graphql > locations.json

jq -r '.data.locationRecords.[] | "\(.label);\(.city),\(.region)"' locations.json | sort > sorted-locations.csv
```

### Finding duplicates

How to find entries in the dump file that have duplicate checksums:

```shell
jq -r '. | "\(.checksum) \(.key)"' assets/dump.json | sort > hashes.txt
cut -d ' ' hashes.txt -f 1 | uniq -d
```
