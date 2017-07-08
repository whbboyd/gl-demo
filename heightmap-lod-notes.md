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

## Selecting a LoD

We would like to have the following properties:

 * LoD increases (resolution decreases) with distance from the camera
 * We (mostly) avoid dropping triangles at the edges of a tile

Note that at distance `d` from the camera, for each triangle to fill approximately the same number of pixels, it should be scaled to `d^2` (because it's notionally on the inside of a spherical shell, with area proportional to the square of radius). This gives us another desirable property:

 * LoD increases approximately quadratically

This doesn't play very well with the second desirable property, which requires that LoD evenly divide tile size (that is, be a power of two, since our tile size is 2^8). We could clamp quadratic LoD values to the next lowest power of two, e.g.

    Quadratic:    1   4   9  16  25  36  49  64  81 100
	Exponential:  1   4   8  16  16  32  32  64  64  64

etc. (Note that we never have a LoD of 2, because the quadratic series grows too fast early on. Should we care?) Or, alternatively, we could pick a nicer tile size with more prime factors, but 256x256 makes optimial use of our index space. (252 has prime factors 2, 3, 7, letting us hit 1, 4, 9, 14, 21, 36, 42, 63, 63, 84—better, but heinous to calculate on the fly and probably not better enough that we should care.)

### Calculating distance of a tile from the camera

Ideally, we'd like to know the distance of the *closest* point on the tile. (If the camera is actually over the tile, LoD should always be 1. Ideally this shouldn't need to be special-cased.) Failing that, either the closest corner or center of the tile seem like reasonable distance targets. Center is easy to calculate and doesn't require any thought, so let's go with that.

We want to scale things so that from the center of one tile to the center of an adjacent tile is distance 2 (so the LoD of the adjacent tile is 4). For this the ideal algorithm seems to be something like

    floor(len(camera - tile_center) / (tile_size * scale)) + 1

Then to get LoD

    min_exp(d^2)

Notable thoughts: if you're not centered in your current tile, the next tile adjacent to the closer side will be rendered at LoD 1. Also, this will act up along diagonals; should we maybe use min-axial distance (that is, min(abs(cam\_x - tile\_x), abs(cam\_z - tile\_z))) rather than Euclidean?

## Updating the LoD

Updating is an expensive operation, so we want to do it as infrequently as possible. (Actually, even more than that, we want to make it efficient, which will ultimately entail updating as few tiles as possible, but that's a project for later.) Thus, we want to regenerate only when LoDs would actually change. (In fact, only when LoDs of *adjacent* tiles would actually change; more distant tiles don't matter nearly as much.) If adjacent tiles are also at LoD 1 (not currently guaranteed by our proposed distance scheme, but can easily be done by dropping it one and flooring at 1 (so the current tile isn't at LoD 0, which will cause problems)), an acceptable approximation is to update on crossing a tile boundary. Alternatively, since that will screw up our scaling, we could update on crossing a *half-tile* boundary.

This simply requires storing in the SimpleHeightmap struct which half-tile (quarter-tile, technically) the LoD is currently centered around and updating if the camera is over a different half-tile.

Alternatively, since that lacks hysteresis (the camera can wander back and forth over a half-tile boundary, triggering an update at each crossover), consider overlapping whole-tile-sized areas on half-tile boundaries, e.g.

    +-----+-----+
    |a    |b    |
	|  +--+--+  |
	|  |c    |  |
	+--+  +  +--+
	|d |     |e |
	|  +-----+  |
	|     |     |
	+-----+-----+

where `c` overlaps with all of `a, b, d, e`. Then if the camera crosses from `a` to `ac`, no update is triggered (since it remains in `a` as well); however, if it then crosses into `bc` an update will occur (since it is no longer in `a`, the previous area of record) into `c` but not if it crosses back into `ac` (since it's still in `c`). Thus, whever crossing a half-tile boundary

 * If the camera is still within the area of record, no update occurs. Track which sub-area (e.g. `ac`) contains the camera.
 * If the camera is out of the area of record, find which area overlaps with the previous sub-area (e.g. if we move from sub-area `ac` to `bc`, the overlap is area `c`), store that as the new area of record, and update LoDs.

This will still break down on diagonal transitions unless we maintain whole-tile-sized areas of record for *every* half-tile sub-area, which results in four overlaps on any given sub-area:

    +-----+--+
	|a |b |  |
	|--+--+--+
	|c |d |  |
	+--+--+--+
	|  |  |  |
	+--+--+--+

The center square in this diagram is part of areas `a`, `b`, `c` and `d`. This makes determining overlap for step 2 difficult, since we may still be overlapping with two areas. It's probably acceptable to chose based on direction of movement (e.g. if we move from area `d` sub-area `abcd` to `ab`, we switch from area `d` to `b` instead of `a` because we're moving north). If there's only one overlap (which will only happen on a diagonal transition), obviously we pick that one.

### Formalizing the above

Given a SimpleHeightmap with scale `s`,

 * An area is defined by its northwest corner (that is, it's corner of minimal `x` and `z`). It has width and height `s` and is evenly divisible by `s/2`.
 * A sub-area is defined by its northwest corner. It has width and height `s/2` and is evenly divisible by `s/2`.
 * Given a sub-area at `sx, sz`, the corresponding overlapping areas are
  * `sx - s/2`, `sz - s/2`
  * `sx - s/2`, `sz`
  * `sx`, `sz - s/2`
  * `sx`, `sz`
 * If the camera is over a new sub-area,
  * If the sub-area does not overlap with the previous area,
   * If the new sub-area is north/south of the previous one, update the area to the next one north/south and update LoDs
   * If the new sub-area is east/west of the previous one, update the area to the next one east/west and update LoDs
   * Otherwise, this is a diagonal transition; update the area to the only one overlapping and update LoDs
  * Update the stored sub-area

Note: remember to multiply all the z-axis values by `ROW_SPACING`.


