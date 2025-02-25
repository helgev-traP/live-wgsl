struct Vertex {
    @location(0) position: vec2<f32>,
};

@vertex
fn vs_main(
    v: Vertex,
) -> @builtin(position) vec4<f32> {
    return vec4<f32>(v.position, 0.0, 1.0);
}
