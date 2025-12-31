# Store Assets in the Raw

- Status: accepted
- Deciders: Nathan Fiedler
- Date: 2020-08-20

## Context

This application has one primary function and that is to store digital assets. There are essentially three different approaches to this problem.

One is to not do anything with the assets, but rather maintain a database with paths to the assets in their original location. There is at least one application that uses this approach, named Unbound, for macOS. While this sounds nice, the application is required to monitor for file system changes and update its database.

A second choice is to move the assets into a blob store, where the address of the asset is the hash digest of the file contents. In some cases, the blob store may split the asset into pieces (as with content-defined chunking) and store the chunks individually. One example of this is [perkeep](https://perkeep.org) which is designed to store any sort of data indefinitely. The store model lends itself well to synchronization with remote blob stores.

An alternative to the split-into-chunks approach is the combine-into-volumes approach, as seen in [SeaweedFS](https://github.com/seaweedfs/seaweedfs). Instead of the assets being split into content-defined chunks, they are combined into a single huge (32 GB) file referred to as a _volume_. This is just as scary as split-into-chunks since any damage to the database means you will never see your assets again.

A third option is to move the assets into a special directory structure, and give the files special names. Apple Photos uses this method when importing photos and videos from a camera or smart phone. This has the benefit of not mangling the files, and yet ensuring that files with the same original name never collide. By "hiding" the assets in this manner, the user is less likely to accidentally remove the files.

## Decision

The idea of chopping assets into pieces and spreading them around an anonymous blob store is rather horrifying, and it seems like most users would find that very unappealing. The other approach, monitoring the file system for changes and updating a database seems very error prone, as the application must rely on the operating system to notify it of every single change that ever occurs.

That leaves the **third option** which has all of the benefits and minimal drawbacks.

The exact method used by the application is to compute a SHA256 digest of the asset, generate a unique identifier, create a database record containing the identifier, original file name, hash digest, and other metadata regarding the file. Then the file is moved into a directory structure that incorporates the date-time of import and the unique identifier. Using this method avoids colliding file names, ensures there are not too many directories and similarly that there are not very many files in a single directory. The path to the asset is then encoded and used as its "full" unique identifier. Thus lookups for the asset are very fast, simply decode the identifier to get the path to the file.

The drawback of this method is that the assets are renamed. On the other hand, the original file names are often not meaningful at all, especially coming from a digital camera or social network web site.

## Consequences

This method of asset storage has been used for many months and has worked very well. It is still working very well in 2025.

Using encoded identifiers for the assets allows for utilizing certain blob stores (Amazon S3 is an example) without any need to map the identifiers to a bucket/object pair (or `fid` as found in SeaweedFS) in the remote store.

## Links

- Perkeep [website](https://perkeep.org)
- SeaweedFS [GitHub](https://github.com/seaweedfs/seaweedfs)
