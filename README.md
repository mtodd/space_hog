# Space Hog

What's hogging all your disk space?

## Benchmarks

### Procedural, linear

```
real	0m16.825s
user	0m7.861s
sys	0m7.740s
```

### Threaded

https://doc.rust-lang.org/std/sync/mpsc/fn.channel.html

dir scanner thread
  * finds directories in a path
  * doesn't have to block these reads on reading file sizes or printing output
file size reader thread
  * reads filesize for a file path
  * doesn't have to block on reading directory contents or printing output
printer thread
  * prints out file path and size
  * doesn't have to walk on reading input