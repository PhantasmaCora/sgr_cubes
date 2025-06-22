
// Vertex shader

@vertex
fn vs_main(
  @builtin(vertex_index) VertexIndex : u32
) -> @builtin(position) vec4f {
  var pos = array(
    vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(-1.0, 1.0),
    vec2(-1.0, 1.0), vec2(1.0, -1.0), vec2(1.0, 1.0),
  );

  return vec4f(pos[VertexIndex], 0.0, 1.0);
}

@group(0) @binding(0)
var t_albedo: texture_2d<f32>;

@group(0) @binding(1)
var t_dsr: texture_2d<f32>;

@group(0) @binding(2)
var t_normal: texture_2d<f32>;

@group(0) @binding(3)
var t_depth: texture_2d<f32>;



struct LightData {
  position: vec3<f32>,
  radius: f32,
  color: vec3<f32>,
  padding: f32
}

struct Lights {
    count: u32,
    @align(16)
    lus: array<LightData>
}

@group(1) @binding(0)
var<storage> lights: Lights;

struct CameraUniform {
    view: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
};
@group(2) @binding(0)
var<uniform> camera: CameraUniform;



fn world_from_screen_coord(coord : vec2f, depth_sample: f32) -> vec3f {
  // reconstruct world-space position from the screen coordinate.
  let posClip = vec4(coord.x * 2.0 - 1.0, (1.0 - coord.y) * 2.0 - 1.0, depth_sample, 1.0);
  let posWorldW = camera.inv_view_proj * posClip;
  let posWorld = posWorldW.xyz / posWorldW.www;
  return posWorld;
}

// Fragment shader
@fragment
fn fs_main(
  @builtin(position) coord : vec4f
) -> @location(0) vec4f {
  var result : vec3f;

  let depth = textureLoad(
    t_depth,
    vec2i(floor(coord.xy)),
    0
  ).x;

  // Don't light the sky.
  if (depth >= 1.0) {
    discard;
  }

  let bufferSize = textureDimensions(t_depth);
  let coordUV = coord.xy / vec2f(bufferSize);
  let position = world_from_screen_coord(coordUV, depth);

  let normal = 2.0 * textureLoad(
    t_normal,
    vec2i(floor(coord.xy)),
    0
  ).xyz - vec3<f32>(1.0);

  let dsr = textureLoad(
    t_dsr,
    vec2i(floor(coord.xy)),
    0
  ).xyz;

  let albedo = textureLoad(
    t_albedo,
    vec2i(floor(coord.xy)),
    0
  ).rgb;

  let viewvec = normalize( camera.view * vec4f(position, 1.0) ).xyz;

  for (var i = 0u; i < lights.count; i++) {
    let L = lights.lus[i].position.xyz - position;
    let dist = length(L);
    if (dist > lights.lus[i].radius) {
      continue;
    }
    let lambert = max(dot(normal, normalize(L)), 0.0);
    result += vec3f(
      dsr.x * lambert * pow(1.0 - dist / lights.lus[i].radius, 2.0) * lights.lus[i].color * albedo
    );

    let half_vec = L - viewvec;
    let specular = max( pow( dot(normal, normalize(half_vec)), 24.0 - 16.0 * dsr.z ), 0.0);

    result += vec3f(
      dsr.y * specular * pow(1.0 - dist / lights.lus[i].radius, 2.0) * lights.lus[i].color * albedo
    );
  }

  // some manual ambient
  //result += vec3(0.05);

  return vec4(result, 1.0);
}
