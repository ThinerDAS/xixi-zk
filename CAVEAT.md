# CAVEAT of xixi-zk

## risczero being unstable, reproducibility threatened

risc0 seems like a stable package, however it is constantly updating (within days its version goes through 2.1.0, 2.2.0, 2.3.0, 2.3.1).

This introduce various problem : the default generated project uses `version = "^2.3.0"` which will fetch whichever version fits 2.3.x . This does not ensure reproducibility.

Also rust 1.88 is stable version at current but may be changed, which might hurt reproducibility.
