# Support remote blob storage

- Status: accepted
- Deciders: Nathan Fiedler
- Date: 2025-12-30

## Context

From the beginning, this project has stored the assets on a locally attached disk. This is all well and good and itself is not an issue. However, due to this decision, certain aspects of the design and implementation run the risk of becoming dependent on the assumption that assets are always stored locally. A design consisting of a blob repository interface and a single concrete implementation does not prove that the design is working.

After encountering [SeaweedFS](https://github.com/seaweedfs/seaweedfs), an open-source remote file store, the opportunity to think about how to design and build a remote blob store for tanuki arose.

One consideration for remote blob stores is that the blobs should be addressable using the existing identifiers. These identifiers happen to be the base64-encoded relative path of the asset. It also happens that these identifiers are unique since a component of the decoded value is a [ULID](https://github.com/ulid/spec) which includes a 48-bit timestamp and 80 bits of randomness. As a result, the existing identifiers _should_ be sufficient for addressing blobs in a remote store, **if** that remote blob store gives the client control over the bucket and object names.

SeaweedFS, while being a very featureful and robust application, has two major drawbacks: 1) all files are combined into 32 GB volumes, and 2) the identifier for retrieving a file is determined by SeaweedFS, not the client. As such, tracking the address for the asset in the blob store would involve adding a record to its database record. While SeaweedFS can generate thumbnails for images, it would involve reading from the database for every search result (or storing the thumbnail URL in the secondary indices).

[Minio](https://www.min.io) could work well, as the asset identifier could be split into two parts, one for the name of the **bucket** and the other for the name of the **object**. However, Minio has some issues including a change to the licensing model. Similar projects such as a [Ceph](https://ceph.io/en/) and [RustFS](https://rustfs.com) could work as well. A drawback of these object stores is that they do not generate thumbnails for images.

Alternatively, writing a new remote blob store based on the old Rust code from this project is very easy -- keep the web server and thumbnail generation code, add a REST API for storing and retrieving blobs, write a unit tests, and it is done in a matter of days. This project is called [namazu](https://github.com/nlfiedler/namazu) and is now available as an alternative to the local blob store implementation in this project.

## Decision

Use newly developed **namazu** project for the initial remote blob storage server.

## Consequences

TBD

## Links

- Namazu [website](https://github.com/nlfiedler/namazu)
