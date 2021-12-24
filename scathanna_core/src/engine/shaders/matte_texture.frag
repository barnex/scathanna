#version 300 es

precision mediump float;

in vec3 f_normal;
in highp vec2 f_tex;
out vec4 out_col;
uniform sampler2D usampler;
uniform vec3 sun_dir;
uniform float ambient;

void main() {
  float light = max(ambient, dot(f_normal, sun_dir));
  vec4 tex = texture(usampler, f_tex);
  out_col = vec4(tex.rgb * light, 1.0);
}
