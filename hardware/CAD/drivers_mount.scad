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

horizontal_spacing = 10;
vertical_spacing = 15;
num_dirvers = 5;
cols = 3;

module circles_bridge_profile(diam, distance, direction = 1) {
    angle = 90 * direction;
    rotate(a = angle) {
        translate([0, diam / 2, 0])
            polygon([
                    [mount_thickness / 2, 0],
                    [-mount_thickness / 2, 0],
                    [-mount_thickness / 2, distance - diam],
                    [mount_thickness / 2, distance - diam],
            ]);
    };
}

module circles_bridge(diam, distance, direction = 1) {
    linear_extrude(height = mount_thickness)
        circles_bridge_profile(diam, distance, direction);
}

module single_mount() {
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
                        circles_bridge_profile(hole_diam, holes_distance, j + 2 * i);
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
}

function driver_grid_pos(num) = [ num % cols, floor(num / cols) ];
function driver_pos(num) = [
        (side_length + horizontal_spacing) * driver_grid_pos(num).x,
        (side_length + vertical_spacing) * driver_grid_pos(num).y
     ];

module vertical_birdges() {
    for (col = [0:1:2]) {
        for (hole_n = [0:1:1]) {
            translate(driver_pos(col))
                translate([hole_side_distance, hole_side_distance, 0])
                translate([holes_distance * hole_n, holes_distance, 0])
                circles_bridge(hole_diam, vertical_spacing + hole_diam + hole_side_distance, 0);
        }
    }
}

module horizontal_bridges() {
    for (num = [0:1:num_dirvers - 1]) {
        grid_pos = driver_grid_pos(num);
        if (!( // skip rightmost mounts
                    (grid_pos.y == 0 && grid_pos.x == 2) ||
                    (grid_pos.y == 1 && grid_pos.x == 1)
             )) {
            for (hole_n = [0:1:1]) {
                translate(driver_pos(num)) {
                    translate([hole_side_distance, hole_side_distance, 0])
                        translate([holes_distance, holes_distance * hole_n, 0])
                        circles_bridge(hole_diam, horizontal_spacing + hole_diam + hole_side_distance, -1);
                };
            }
        }
    }
}

module bridges() {
    vertical_birdges();
    horizontal_bridges();
}

module last_bar() {
    translate(driver_pos(4))
        translate([hole_side_distance, hole_side_distance, 0])
        translate([holes_distance, 0, 0])
        circles_bridge(hole_diam, side_length + hole_diam * 2 + wall_thickness * 3 - wall_thickness / 2 + tolerance, -1);
}

for (num = [0:1:num_dirvers - 1]) {
    translate(driver_pos(num))
        single_mount();
}

bridges();
last_bar();

