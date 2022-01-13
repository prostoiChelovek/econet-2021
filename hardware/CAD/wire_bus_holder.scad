tolerance = 0.4;

support_thickness = 3;
vertical_gap = 3;

guides_spacing = 35;

wall_width = 1.6;
wall_height = 2;
wall_depth = 3;
gap_witdh = 1.2 + tolerance;
wires_num = 2;
supports_num = 2;

walls_num = wires_num + 1;
total_width = wall_width * walls_num + gap_witdh * wires_num;

support_offset = support_thickness + vertical_gap;

module single_guide() {
    for (n = [0:1:wires_num]) {
        translate([0, (wall_width + gap_witdh) * n])
            cube(size = [wall_depth, wall_width, wall_height]);
    }

    translate([0, 0, -support_offset])
        cube(size = [wall_depth, total_width, support_offset]);
}


for (n = [0:1:supports_num - 1]) {
    translate([(wall_width + guides_spacing) * n, 0, 0])
        single_guide();
}

for (n = [0:1:supports_num - 2]) {
    translate([(wall_width + guides_spacing) * n, 0, -support_offset])
        cube(size = [wall_depth + guides_spacing, total_width, support_thickness]);
}

