# pugl-ui – a stub for small embeddable GUI-toolkits using pugl

pugl is a minimal portable API for embeddable GUIs https://gitlab.com/lv2/pugl/

This crate aims to provide a stub for GUI-toolkits using pugl


## Goal

Stub for small light weight self contained GUI toolkits, especially to
implement UIs of [LV2 plugins](https://lv2plug.in). GUIs for LV2 plugins need
to be self contained, i.e. they should be statically linked and must not
dynamically link any other GUI toolkit. Otherwise symbols of the same GUI
toolkit in different versions used by different plugins running in the same
host would clash.

There is the crate `pugl-sys` which is the wrapper around the `pugl`
library. This crate provides widget layout and event propagation.

It does not, however, provide the widgets themselves. In the LV2 world many
plugin authors write their own widget sets with their distinct look and feel as
they are part of the authors corporate identity.

## Status

Early prototype stage. Currently only tested on Linux/X11


## How to use

This is not a crate distributed on https://crates.io, yet. There's still some
manual work to do to use and test it.

### Prerequisites

Clone and build the crate [pugl-sys](https://github.com/johannes-mueller/pugl-sys) next
to where you cloned this one to.


### Build

* Run `cargo build`

* Run `cargo test`. A funny looking window should appear.

* There is one example "app" that you can call by `cargo run
  --example=widgets`. It's three dial nobs that you can turn by the mouse
  wheel. By clicking "Reset" they go back to their original position.


## Todo

Still a lot.

* Lots of code simplifications.

* Documentation

* For sure a lot mode
