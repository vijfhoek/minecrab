[[block]]
struct Uniforms {
    view_position: vec4<f32>;
    view_projection: mat4x4<f32>;
};

[[block]]
struct Light {
    position: vec3<f32>;
    color: vec3<f32>;
};

[[group(1), binding(0)]]
var<uniform> uniforms: Uniforms;

[[group(2), binding(0)]]
var<uniform> light: Light;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] texture_coordinates: vec2<f32>;
    [[location(2)]] normal: vec3<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] texture_coordinates: vec2<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] world_position: vec3<f32>;
};

struct InstanceInput {
    [[location(5)]] model_matrix_0: vec4<f32>;
    [[location(6)]] model_matrix_1: vec4<f32>;
    [[location(7)]] model_matrix_2: vec4<f32>;
    [[location(8)]] model_matrix_3: vec4<f32>;
    [[location(9)]] texture_index: u32;
};


[[stage(vertex)]]
fn main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexOutput;
    out.texture_coordinates = model.texture_coordinates;
    out.world_normal = model.normal;

    var world_position: vec4<f32> = model_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;

    out.clip_position = uniforms.view_projection * model_matrix * vec4<f32>(model.position, 1.0);

    return out;
}

[[group(0), binding(0)]] var sampler_diffuse: sampler;
[[group(0), binding(1)]] var texture: texture_2d<f32>;

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let object_color: vec4<f32> =
        textureSample(texture, sampler_diffuse, in.texture_coordinates);

    let ambient_strength = 0.2;
    let ambient_color = light.color * ambient_strength;

    let light_direction = normalize(light.position - in.world_position);
    let view_direction = normalize(uniforms.view_position.xyz - in.world_position);
    let half_direction = normalize(view_direction + light_direction);

    let diffuse_strength = max(dot(in.world_normal, light_direction), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    let specular_strength = pow(max(dot(in.world_normal, half_direction), 0.0), 32.0);
    let specular_color = specular_strength * light.color;

    let result = (ambient_color + diffuse_color + specular_color) * object_color.xyz;
    return vec4<f32>(result, object_color.a);
}
