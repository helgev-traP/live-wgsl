struct Info {
    viewport_size: vec2<f32>,
    time_from_start_up: f32,
    time_from_update: f32,
}

@group(0) @binding(0)
var<uniform> info: Info;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let clip_position = (position.xy / info.viewport_size) * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0);

    var color = vec3<f32>(0.0, 0.0, 0.0);

    let waves = 30;
    let speed = 0.5;
    let amplitude = .8;
    let line_width = 0.005;

    for (var i = 0; i < waves; i = i + 1) {
        let c = hsl_to_rgb(
            360.0 / f32(waves) * f32(i),
            1.0,
            0.5,
        );

        let speed = speed * (1.0 + 1.0 / f32(waves) * f32(i));

        let wave = sin(clip_position.x + speed * info.time_from_update) * (amplitude / f32(waves) * f32(i) + 0.1);
        let distance = abs(clip_position.y - wave);

        let line = smoothstep(0.0, line_width, distance);
        color += (1.0 - line) * c;
    }

    return vec4<f32>(color, 1.0);
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> vec3<f32> {
    let c = (1.0 - abs(2.0 * l - 1.0)) * s;
    let x = c * (1.0 - abs((h / 60.0) % 2.0 - 1.0));
    let m = l - c / 2.0;

    var rgb = vec3<f32>(0.0, 0.0, 0.0);

    if h < 60.0 {
        rgb = vec3<f32>(c, x, 0.0);
    } else if h < 120.0 {
        rgb = vec3<f32>(x, c, 0.0);
    } else if h < 180.0 {
        rgb = vec3<f32>(0.0, c, x);
    } else if h < 240.0 {
        rgb = vec3<f32>(0.0, x, c);
    } else if h < 300.0 {
        rgb = vec3<f32>(x, 0.0, c);
    } else {
        rgb = vec3<f32>(c, 0.0, x);
    }

    return rgb + m;
}