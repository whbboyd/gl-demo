# Heightmap implementation notes

A heightmap is a way of storing terrain which is compact, easy to manipulate, and easy to render. It represents ground as a single mesh, uniform in XZ and displaced on Y, with only the Y displacement stored (as XZ is uniform and easily inferred).

## Possible representations

### Quads:

    o-o-o
    | | |
    o-o-o
    | | |
    o-o-o

Note that each quad is actually two tris:

    o-o-o
    |/|/|
    o-o-o
    |/|/|
    o-o-o

and they are normally broken over that seam.

Advantages:

 * Easy to convert heightmap index to mesh coordinates 

Disadvantages:

 * Implicit tris are not very uniform
 * Very artifact-ey at extreme slope changes
 * Implicit tris can result in inconsistent results unless triangulation convention is enforced

### Tris:

    o---o---o
     \ / \ /
      o---o
     / \ / \
    o---o---o

Advantages:

 * Tris are explicit
 * Minimal artifacting
 * The most uniform possible polygons (equilateral triangles on flat surfaces)

Disadvantages:

 * Converting heightmap index to mesh coordinates is more complicated

The winner is: tris! We're sure not afraid of math here, we're doing 3D rendering…

## Converting heightmap index to mesh XZ

### Internal heightmap indexing

Because the vertices don't fall on a Cartesian grid, conversion from heightmap index to vertex XZ is more complicated than `index x,y * scale → vertex x,z`. The obvious thing to do is index vertices left-to-right, top-to-bottom, e.g.

    0---1---2               0-1-2
     \ / \ / \  (Stored as) |/|/|
      3---4---5     -->     3-4-5
     / \ / \ /              |\|\|
    6---7---8               6-7-8

Downsides to doing this include that the change in Z from 0 to 3 is less than the change in X from 0 to 1, by

    .5 * tan(pi/3) ~ 0.866

Also, every other row is offset on Y by 0.5.

Thus, for a tri mesh `m` vertices wide, the vertex at index `n` has XZ

    X = ((n % m) + (if (n / m) % 2 == 0 then 0 else 0.5)) * scale
    Z = (n / m) * 0.8660254037844386 * scale

Not too bad! The whole mesh is `(m + 0.5) * scale` wide (assuming `m > 2n`) and `n/m * 0.866… * scale` high.

## Normals

Given a vertex v and surrounding vertices 0-5:

      0   1
       \ /
    2---v---3
       / \
      4   5

the normal at v should be approximately the average of the perpendicular to each incident edge.

A few useful values:

 * Slope between adjacent vertices `u` and `v` is simply `(u.Y - v.Y) / scale`, because all vertices are `scale` units apart.
 * The normal at `v` is the sum of the perpendiculars to `0v`, `1v`, etc., divided by 6. (This may not be the best way to calculate it. Also, there aren't always six adjacent vertices near the edge.)
 * Want to rotate around [z, 0, x]
 * Rodrigues' formula:
	* `v_rot = v cos theta + (axis cross v) sin theta + axis(axis dot v)(1 - cos theta)`
	* Simplifies: `v_rot = axis cross v + axis(axis dot v)`
		* `= [a_y v_z - a_z v_y, a_z v_x - a_x v_z, a_x v_y - a_y v_x] + …`
		* `= [-a_z v_y, a_z v_x - a_x v_z, a_x v_y] + …`
		* `… = axis (a_x v_x + a_z v_z)

## Texturing

It's mostly possible for us to tile a rectangular texture over our triangle grid. Advantage of this is no wasted texture data and it's reasonably possible to tile square textures over our heightmap. Disadvantage is that texture mapping is slightly harder and not all tris have the same texture map.

Another consideration worth considering: depending on slope, not every heightmap tri has the same surface area (to say nothing of shape). Trying to automate this and maintain tiling is hopeless, though.

Far and away the easiest option is to map texture u,v to vertex x,z with appropriate wrapping. The major downside here is that no two tris have the same texture mapped to them. Do we care? (Signs point to no.)

# Heightmap physics notes

Heightmap collision is relatively easy; find the tri directly below the collision point, calculate height based on its vertices, and do regular ground clipping to that height.

## Finding a heightmap tri

We need a way to go from x,z to the three vertices of a triangle. Right now, the heightmap itself doesn't know enough to do this because the main method moves it around with a model matrix. So, we'll have the heightmap take an offset in x/z and apply that to each vertex.

Going from 3D coordinates to heightmap tri is now a matter of inverting the index-to-vertex transformation:

 * `x = (idx % width) + (idx % 2 == 0 ? 0 : 0.5) * resolution + x_off`
	* → `(x - x_off) / resolution = idx % width + (idx % 2 == 0 ? 0 : 0.5)`
	* → `(x - x_off) / resolution = idx`
 * `z = idx / width * 0.866… * resolution + z_off`
	* → `(z - z_off) / resolution / 0.866… * width = idx

Within bounds, floor this and we won't be more than one vertex off. (Need to deal with the parity offset on x, I think that's the biggest error.) Out of bounds…? We can detect this, so we could just ignore it, or set a default floor.


