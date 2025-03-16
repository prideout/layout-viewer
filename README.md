## Tasks

Add the log crate and use it everywhere that we're currently using print (except in main)

Get it work in desktop app.  No need for mouse interaction yet.

Create a web app with trunk and Yew. It should have two routes: a home route and a layouts/{id}
route.

On the layout page, there should be a sidebar on the right that shows a list of
layers. Each layer has a color swatch, visibility toggle, and opacity slider.
The central area of the app should be a filled a GL canvas that we can render to
using glow and our `GlRenderer` class.  Above the layer list are a few buttons:
"Back home", "Enable picking", "Show all", and "Hide all".

Add BSD 3-clause and open source it.

Use [https://docs.rs/crate/bvh](https://docs.rs/crate/bvh) for accelerated
picking. It uses nalgebra internally.

Save array refs for last.

## References

- https://www.artwork.com/gdsii/gdsii/page5.htm
- https://github.com/GraphiteEditor/Graphite/blob/master/node-graph/gcore/src/graphic_element/renderer.rs
- https://crates.io/crates/gds21

## Web app references

- https://jakearchibald.github.io/svgomg/
- https://github.com/bumbu/svg-pan-zoom
- https://docs.rs/specta/latest/specta/
- https://github.com/jakearchibald/svgomg/blob/1e1a1448f25761e7382cae5de2ba21f1e6ba439d/src/css/_global.scss#L16

