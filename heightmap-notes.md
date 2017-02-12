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
 * Given a vec [x, y, z], the perpendicular towards [0, 1, 0] is
	* if y is 0, it's just [0, 1, 0].
	* if y is positive, it's [-x, 1/y, -z]
	* if y is negative, it's [x, 1/y, z]

