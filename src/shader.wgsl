//[[block]]
struct VertexInput {
	[[location(0)]] position: vec3<f32>;
};

//[[block]]
struct InstanceTransform {
  [[location(10)]] a: vec4<f32>;
  [[location(11)]] b: vec4<f32>;
  [[location(12)]] c: vec4<f32>;
  [[location(13)]] d: vec4<f32>;
};

struct VertexOutput {
	[[builtin(position)]] position: vec4<f32>;
};

//[[block]]
struct RotUniform {
	rot: mat4x4<f32>;
};
//[[block]]
struct CamUniform {
	rot: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> rot: RotUniform;
[[group(0), binding(1)]]
var<uniform> camera: CamUniform;

[[stage(vertex)]]
fn vs_main(in: VertexInput, instance: InstanceTransform) -> VertexOutput {
	var out: VertexOutput;
	var transform = mat4x4<f32>(
		instance.a,
		instance.b,
		instance.c,
		instance.d,
	);
	out.position = camera.rot * transform * vec4<f32>(in.position.xyz, 3.0);
	return out;
}

[[stage(fragment)]]
fn fs_main([[builtin(position)]] coord: vec4<f32>) -> [[location(0)]] vec4<f32> {
  return vec4<f32>(coord.xyz, 1.0);
}
