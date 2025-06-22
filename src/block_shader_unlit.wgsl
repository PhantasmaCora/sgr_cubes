
// Vertex shader

struct CameraUniform {
    view: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) ao: f32,
    @location(4) metalrough: i32,
    @location(5) tex_index: u32
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) ao: f32,
    @location(2) normal: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.color = model.color;
    out.ao = model.ao;
    out.normal = model.normal;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    var bri = min(1.0, max(0.0, dot(in.normal, vec3<f32>(0.5547, 0.8321, 0.0) ) ));
    bri = 0.5 * bri + 0.5;

    var out_color: vec4<f32>;

    out_color.r = in.color.r * in.ao * bri;
    out_color.g = in.color.g * in.ao * bri;
    out_color.b = in.color.b * in.ao * bri;
    out_color.a = 1.0;

    return out_color;
}
