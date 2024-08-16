
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
    @location(1) array_index: u32,
    @location(2) light: f32
};

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = model.uv;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.array_index = model.array_index;
    out.light = model.light;
    return out;
}

// Fragment shader

@group(1) @binding(0)
var t_diffuse: texture_2d_array<u32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@group(2) @binding(0)
var t_lightmap: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureLoad( t_diffuse, vec2<u32>( u32( in.uv[0] * f32(textureDimensions(t_diffuse, 0)[0]) ), u32( in.uv[1] * f32(textureDimensions(t_diffuse, 0)[1]) ) ), in.array_index, 0);
    if (color.r == 255) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    let light_row: u32 = 7 - u32( floor( in.light * 7.0 + 0.5 ) );
    return textureLoad( t_lightmap, vec2<u32>( color.r, light_row ), 0 );
}
