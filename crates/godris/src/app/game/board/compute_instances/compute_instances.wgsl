#import indirect_instancing::instance_struct

struct BoardInstanceData {
    base: InstanceData;
    color: vec4<f32>;
};

struct BoardInstances {
    instances: array<BoardInstanceData>;
};

struct Uniforms {
    board_size: vec3<u32>;
    _: u32;
    board_visible_min: vec3<u32>;
    _: u32;
    board_visible_max: vec3<u32>;
    _: u32;
};

struct Cell {
    color: vec4<f32>;
};

struct Cells {
    cells: array<Cell>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

[[group(1), binding(0)]]
var<storage, read> in_cells: Cells;

[[group(1), binding(1)]]
var<storage, read_write> out_instances_opaque: BoardInstances;

[[group(1), binding(2)]]
var<storage, read_write> out_instances_transparent: BoardInstances;

fn index_to_pos(idx: u32, board_size: vec3<u32>) -> vec3<u32> {
    return vec3<u32>(
        idx % board_size.x,
        (idx / board_size.x) % board_size.y,
        (idx / (board_size.x * board_size.y)) % board_size.z,
    );
}

fn pos_to_index(pos: vec3<u32>, board_size: vec3<u32>) -> u32 {
    return ((pos.z * board_size.y * board_size.x) + (pos.y * board_size.x) + pos.x);
}

[[stage(compute), workgroup_size(64)]]
fn instances([[builtin(global_invocation_id)]] invocation_id: vec3<u32>) {
    // Fetch board size
    let board_size = uniforms.board_size;
    let visible_size = uniforms.board_visible_max - uniforms.board_visible_min;

    // Calculate maximum indices
    let max_cell = arrayLength(&in_cells.cells);

    // Destructure invocation index
    var cell_index = invocation_id.x;

    // Early-out if we're out of bounds
    if (cell_index >= max_cell) {
        return;
    }

    // Convert index to position
    var cell_position = index_to_pos(cell_index, visible_size);

    // Offset by visible minimum
    cell_position = cell_position + uniforms.board_visible_min;

    // Recalculate cell index to account for position change
    cell_index = pos_to_index(cell_position, board_size);

    // Fetch cell
    let cell = in_cells.cells[cell_index];
    let translation = vec3<f32>(cell_position) - vec3<f32>(board_size) * 0.5;

    // Use scale to hide fully transparent cells
    if (cell.color.w < 1.0) {
        let transform = mat4x4<f32>(
            vec4<f32>(1.0, 0.0, 0.0, 0.0),
            vec4<f32>(0.0, 1.0, 0.0, 0.0),
            vec4<f32>(0.0, 0.0, 1.0, 0.0),
            vec4<f32>(translation,   1.0),
        );

        // Write transparent cell
        out_instances_transparent.instances[cell_index].base.transform = transform;
        out_instances_transparent.instances[cell_index].color = cell.color;
    }
    else {
        let scale = step(1.0 - cell.color.w, 0.0);
        let transform = mat4x4<f32>(
            vec4<f32>(scale, 0.0,   0.0,   0.0),
            vec4<f32>(0.0,   scale, 0.0,   0.0),
            vec4<f32>(0.0,   0.0,   scale, 0.0),
            vec4<f32>(translation,  scale),
        );
    
        // Calculate shadow
        var color = cell.color;
        for (var y = cell_position.y + 1u; y < board_size.y; y = y + 1u) {
            let target_index = pos_to_index(
                vec3<u32>(cell_position.x, y, cell_position.z),
                board_size
            );
            let target_cell = in_cells.cells[target_index];
            if (target_cell.color.w >= 1.0) {
                color = vec4<f32>(color.xyz * 0.5, color.w);
                break;
            }
        }

        // Write opaque cell
        out_instances_opaque.instances[cell_index].base.transform = transform;
        out_instances_opaque.instances[cell_index].color = color;
    }
}
