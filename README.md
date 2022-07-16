# elfpromote

A small utility for modifying ELF shared library loading order.

## Usage

```bash
$ cargo install elfpromote
$ ldd blueboat_server
        linux-vdso.so.1 (0x00007ffe4597b000)
        libsqlite3.so.0 => /lib/x86_64-linux-gnu/libsqlite3.so.0 (0x00007fc6c5df2000)
        libseccomp.so.2 => /lib/x86_64-linux-gnu/libseccomp.so.2 (0x00007fc6c5dd0000)
        libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x00007fc6c5bde000)
        /lib64/ld-linux-x86-64.so.2 (0x00007fc6c91ac000)
        libfdb_c.so => /lib/libfdb_c.so (0x00007fc6c47ed000)
        libz.so.1 => /lib/x86_64-linux-gnu/libz.so.1 (0x00007fc6c47d1000)
        ...
$ elfpromote ./blueboat_server -o blueboat_server.promoted --lib libfdb_c.so
$ ldd blueboat_server.promoted 
        linux-vdso.so.1 (0x00007fffba59e000)
        libfdb_c.so => /lib/libfdb_c.so (0x00007f06257a1000)
        libsqlite3.so.0 => /lib/x86_64-linux-gnu/libsqlite3.so.0 (0x00007f0625678000)
        libseccomp.so.2 => /lib/x86_64-linux-gnu/libseccomp.so.2 (0x00007f0625656000)
        libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x00007f0625464000)
        /lib64/ld-linux-x86-64.so.2 (0x00007f0629e23000)
        libz.so.1 => /lib/x86_64-linux-gnu/libz.so.1 (0x00007f0625448000)
        ...
```
