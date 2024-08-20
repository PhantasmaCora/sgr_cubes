
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
};

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = model.uv;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader
@group(1) @binding(0)
var t_select: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pos = vec2<u32>( u32( in.uv[0] * f32(textureDimensions(t_select, 0)[0]) ), u32( in.uv[1] * f32(textureDimensions(t_select, 0)[1]) ) );
    return vec4<f32>(0.867, 0.584, 0.047, textureLoad( t_select, pos, 0 ).a );
}
