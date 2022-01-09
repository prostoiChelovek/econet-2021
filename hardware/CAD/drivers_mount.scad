$fa = 1;
$fs = 0.4;

side_length = 43.5;
hole_diam = 3;
holes_distance = 40 - hole_diam;
hole_side_distance = (side_length - holes_distance) / 2;

module holes_pattern() {
    translate([hole_side_distance, hole_side_distance, 0])
        for (nx = [0: 1: 1]) {
            for (ny = [0: 1: 1]) {
                translate([holes_distance * nx, holes_distance * ny, 0])
                    children();
            }
        }

}

difference() {
    square([side_length, side_length]);

    holes_pattern() {
        circle(d = hole_diam);
    }
}
