$fa = 1;
$fs = 0.4;

tolerance = 0.4;
hole_diam = 3;
holes_distance = 20;
thickness = 3;

distnace_to_drawer = 29;

base_width = hole_diam + holes_distance + thickness * 2;
base_height = hole_diam + thickness * 2;

difference() {
    cube(size = [base_width, base_height, thickness]);
    for (i = [0:1:1]) {
        translate([thickness + hole_diam / 2 + holes_distance * i, thickness + hole_diam / 2, -1])
            cylinder(d = hole_diam + tolerance, h = thickness + 2);
    }
}

translate([thickness + hole_diam + holes_distance / 2 - thickness, 0, 0])
    cube(size = [thickness, distnace_to_drawer - 2, 20 + thickness]);

