
// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) array_index: u32,
    @location(3) light: f32
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) array_index: u32
};

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = model.uv;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.array_index = model.array_index;
    return out;
}

// Fragment shader

@group(1) @binding(0)
var t_diffuse: texture_2d_array<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample( t_diffuse, s_diffuse, in.uv, in.array_index );
}
