struct VertexInput {
    [[location(0)]] position: vec2<f32>;
    [[location(1)]] texture_coordinates: vec2<f32>;
    [[location(2)]] texture_index: i32;
    [[location(3)]] color: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] texture_coordinates: vec2<f32>;
    [[location(1)]] texture_index: i32;
    [[location(2)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.texture_coordinates = model.texture_coordinates;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    out.texture_index = model.texture_index;
    out.color = model.color;
    return out;
}

[[group(0), binding(0)]] var sampler: sampler;
[[group(0), binding(1)]] var texture: texture_2d_array<f32>;

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return textureSample(texture, sampler, in.texture_coordinates, in.texture_index)
        * in.color;
}
