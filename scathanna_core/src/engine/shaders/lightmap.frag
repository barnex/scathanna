#version 300 es

precision highp float;

in vec3 f_normal;
in highp vec2 f_tex;
in highp vec2 f_lightmap;
out vec4 out_col;
uniform sampler2D texture_unit;
uniform sampler2D lightmap_unit;
uniform vec3 sun_dir;
uniform float ambient;

void main() {
  // float light = max(ambient, dot(f_normal, sun_dir));
  vec4 light = texture(lightmap_unit, f_lightmap);
  vec4 tex = texture(texture_unit, f_tex);
  out_col = vec4(tex.rgb * light.rgb, 1.0);
}
