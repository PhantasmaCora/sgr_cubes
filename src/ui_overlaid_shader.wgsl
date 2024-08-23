
// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) uv2: vec2<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) uv2: vec2<f32>
};

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = model.uv;
    out.uv2 = model.uv2;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_base: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@group(1) @binding(0)
var t_over: texture_2d<f32>;
@group(1) @binding(1)
var s_over: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color_a = textureSample( t_base, s_diffuse, in.uv );
    var color_b = textureSample( t_over, s_over, in.uv2 );
    var inter = color_b.a;
    return vec4<f32>( inter * color_b.r + (1.0 - inter) * color_a.r, inter * color_b.g + (1.0 - inter) * color_a.g, inter * color_b.b + (1.0 - inter) * color_a.b, color_a.a);
}
