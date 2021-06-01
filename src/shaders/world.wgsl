[[block]]
struct View {
    position: vec4<f32>;
    projection: mat4x4<f32>;
};

[[block]]
struct Time {
    time: f32;
};

[[group(1), binding(0)]]
var<uniform> view: View;

[[group(2), binding(0)]]
var<uniform> time: Time;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] texture_coordinates: vec2<f32>;
    [[location(2)]] normal: vec3<f32>;
    [[location(3)]] highlighted: i32;
    [[location(4)]] texture_id: i32;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] texture_coordinates: vec2<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] world_position: vec3<f32>;
    [[location(3)]] highlighted: i32;
    [[location(4)]] texture_id: i32;
};

let pi: f32 = 3.14159265359;

[[stage(vertex)]]
fn main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.world_normal = model.normal;
    if (model.texture_id == 8) {
        // water
        let offset = (sin(time.time * 0.5 + model.position.x) * cos(time.time * 0.9 + model.position.y) + 2.5) / 10.0;
        out.world_position = vec3<f32>(model.position.x, model.position.y - offset, model.position.z);
        out.texture_coordinates = model.texture_coordinates + (time.time / 10.0);
        out.texture_id = i32(8.0 + (time.time * 10.0) % 32.0);
    } else {
        out.world_position = model.position;
        out.texture_coordinates = model.texture_coordinates;
        out.texture_id = model.texture_id;
    }

    out.clip_position = view.projection * vec4<f32>(out.world_position, 1.0);
    out.highlighted = model.highlighted;
    return out;
}

[[group(0), binding(0)]] var texture_sampler: sampler;
[[group(0), binding(1)]] var texture_array: texture_2d_array<f32>;

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let object_color: vec4<f32> = textureSample(
        texture_array,
        texture_sampler,
        in.texture_coordinates,
        in.texture_id
    );

    let light_position = vec3<f32>(-100.0, 500.0, -200.0);
    let light_color = vec3<f32>(1.0, 1.0, 1.0);

    let ambient_strength = 0.1;
    let ambient_color = light_color * ambient_strength;

    let light_direction = normalize(light_position - in.world_position);
    let view_direction = normalize(view.position.xyz - in.world_position);
    let half_direction = normalize(view_direction + light_direction);

    let diffuse_strength = max(dot(in.world_normal, light_direction), 0.0);
    let diffuse_color = light_color * diffuse_strength;

    let specular_strength = pow(max(dot(in.world_normal, half_direction), 0.0), 32.0);
    let specular_color = specular_strength * light_color;

    var result: vec3<f32> = (ambient_color + diffuse_color + specular_color) * object_color.xyz;
    if (in.highlighted != 0) {
        result = result + 0.25 + sin(time.time * pi) * 0.07;
    }

    return vec4<f32>(result, object_color.a);
}
