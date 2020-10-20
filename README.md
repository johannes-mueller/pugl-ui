# pugl-ui – a stub for small embeddable GUI-toolkits using pugl

[![Build Status][travis-badge]][travis-url] [![Current Crates.io Version][crates-badge]][crates-url]

[travis-badge]: https://travis-ci.com/johannes-mueller/pugl-ui.svg?branch=master
[travis-url]: https://travis-ci.com/johannes-mueller/pugl-ui
[crates-badge]: https://img.shields.io/crates/v/pugl-ui.svg
[crates-url]: https://crates.io/crates/pugl-ui

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

Kind of beta testing stage. Currently only tested on Linux/X11

### API stability

Before reaching the 1.0.0 release, incompatible API changes can happen. There
is no large base of applications using `pugl-ui` as of yet. So experience with
the API is limited. The 1.0.0 release will not happen before several developers
have used `pugl-ui` for real life applications and given feedback.

If it turns out that there is a better way to design the API it will be
done. It is somewhat likely, that the API will change towards more convenient
use, as more practical experience is collected.

Especially because the API differs from usual object oriented approaches of GUI
programming. The design patterns used in object oriented GUI programming, most
prominently the observer pattern and model view controller, don't work in
proper safe Rust, as shared mutable references are needed to implement them.

## Todo

* More complete documentation

However I am reluctant to put too much effort into documenting when the API is
not stable. So first I will write some LV2 Plugins to see if everything works
convenient. So if there is documentation missing to a certain module, chances
are, that the API will change.


## Applications using `pugl-ui`

* [Envolvigo](https://github.com/johannes-mueller/envolvigo)
