# special-zip
(Not for common use) a specialized ZIP archiver for Wikipedia on Swarm project

It recursively stores in a ZIP-file a directory,
compressing files with Gzip but marking them uncompressed.
To each file (except of directories) it adds 32 zero bytes
extra data of type Record Management Controls.
