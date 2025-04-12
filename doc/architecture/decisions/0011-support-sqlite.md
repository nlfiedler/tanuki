# Use RocksDB

* Status: accepted
* Deciders: Nathan Fiedler
* Date: 2025-03-13

## Context

Asset metadata is stored in a database and that has been in [RocksDB](https://rocksdb.org) for several years. In the interest of exploring alternative databases, support should be added to offer a choice of implementations at deployment time.

## Decision

[SQLite](https://sqlite.org/index.html) is small, fast, and reliable (according to the web site) and works very well for this type of application.

### Drawbacks

Implementing indexes on complex values like a list of strings or a compound structure is not easy (or possible?), but with a small enough number of assets (tens of thousands), this is not an issue that anyone will notice.

SQLite lacks support for date/time and compound values. Representing a list of strings in a single column requires serializing the list in an application-specific manner (e.g. tab-separated values).

### Advantages

An advantage of SQLite is that the database can be queried and updated via the command line utility, unlike a RocksDB instance that has no schema or command-line tools.

### Conclusion

The initial alternative data store will be SQLite.

## Consequences

Disk usage is a bit less than RocksDB, but load time of 21,000 assets is 4.5 minutes versus 4 seconds. This is likely due to the need to normalize the asset and location records, which involves a query to ensure no duplicates are inserted.

## Links

* SQLite [website](https://sqlite.org)
* rusqlite [GitHub](https://github.com/rusqlite/rusqlite)
