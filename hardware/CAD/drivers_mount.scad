$fa = 1;
$fs = 0.4;

tolerance = 0.4;
mount_thickness = 4;
wall_thickness = 2.6;
mount_height = 30;

side_length = 43.5;
hole_diam = 3 + tolerance;
holes_distance = 40 - hole_diam;
hole_side_distance = (side_length - holes_distance) / 2;

module circles_bridge(diam, distance, thickness, direction = 1) {
    angle = 90 * direction;
    rotate(a = angle) {
        translate([0, diam / 2, 0])
            polygon([
                    [thickness / 2, 0],
                    [-thickness / 2, 0],
                    [-thickness / 2, distance - diam],
                    [thickness / 2, distance - diam],
            ]);
    };
}

module holes_pattern() {
    translate([hole_side_distance, hole_side_distance, 0])
        for (nx = [0: 1: 1]) {
            for (ny = [0: 1: 1]) {
                translate([holes_distance * nx, holes_distance * ny, 0])
                    children();
            }
        }

}

module standoff_profile() {
    difference() {
        circle(d = hole_diam + wall_thickness * 2);
        circle(d = hole_diam);
    }
}

module standoff() {
    linear_extrude(height = mount_height + mount_thickness)
        standoff_profile();
}

module standoffs() {
    holes_pattern() {
        standoff();
    };
}

module standoff_bridges_profile() {
    translate([hole_side_distance, hole_side_distance, 0]){
        for (i = [0:1:1]) {
            translate([holes_distance * i, holes_distance * i, 0]) {
                for (j = [-1:1:0]) {
                    circles_bridge(hole_diam, holes_distance, 4, j + 2 * i);
                }
            };
        }
    }
}

module standoff_bridges() {
    linear_extrude(height = mount_thickness)
        standoff_bridges_profile();
}

standoffs();
standoff_bridges();

