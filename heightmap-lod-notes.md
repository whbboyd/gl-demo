# Heightmap LoD

We want to vary heightmap resolution based on distance. Easiest way to do this is probably just to line up and swap out varying-resolution heightmaps as needed. (A fun wrinkle: our heightmaps are rectangular, despite being meshes of equilateral triangles, so the edges will be fun to line up.)

Our approach of choice is to have a rectangular grid of tiles at varying resolutions (but all the same size). Seams are TBD.

## API for managed, LoD-tiled heightmaps

We need to be able to

 * Turn an array of tiles into one or more renderable meshes
 * Swap out low- and high-resolution tiles on a frame-by-frame basis with high performance

Based on this, it seems like we need to

 * Provide an iterable over internally-managed gpu::Models
 * Provide a method to update LoD based on physical location

**BORROW CHECKER ALERT** Nope. The lifetimes immediately become unmanageable because it's hard to tell the borrow checker that the renderer doesn't borrow these objects for very long. Thus, we end up with the following:

 * A `Renderable` trait with a `render` method, implemented for `gpu::ModelInstance`
 * An implementation of `Renderable` for `SimpleHeightmap` which turns the heightmap into a `ModelInstance` and calls `render` on it.

Thus for our in-memory LoD heightmap, we want

 * A high-resolution `SimpleHeightmapGeometry` providing physics and a source for tiles
 * A way to cut that `SimpleHeightmapGeometry` into tiles for rendering
 * A way to scale the resolution of tiles
 * A `Renderable` implementation that iterates over all tiles

Stuff shared with `SimpleHeightmap`:

 * Actually, everything except `model` and `Heightmap.update_lod`

Conclusion: stuff the idea of keeping them seperate, there's no good reason to.

A few useful assumptions to make:

 * We'll make tiles square-ish. (The fact that they're rectangular triangle meshes screws with this. They'll have the same number of rows and columns, which makes the physical geometry about 1:.866.)
	* The default tile size is 256x256. This allows us to downgrade our indices to u16, saving *massive boatloads* of memory.
