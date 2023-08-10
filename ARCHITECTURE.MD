# readDB Architecture Overview

The database consists of 3 parts:

1. The index table
2. The data file(s)
3. The cache

### Index Table
The index table is an in-memory kv store that maps keys to the line number where they start in the respective files.
The memory footprint here is key-size + 8 bytes (for the line number). The index table is persisted to disk when persist
is called. While the database is immutable, the index table allows for linking of new keys to already preexisting data
through the use of the `link` function. Therefore the index table needs to be persisted to disk.

### Data file(s)
The data file(s) are the actual data. They are immutable and are never modified. The data file(s) are lazily loaded and 
kept open for the lifetime of the database.

### Cache
The cache is again in-memory and is used to speed up reads. Various strategies exist, but the default is a simple LFU
cache. The cache is not persisted to disk and is therefore lost on restart.