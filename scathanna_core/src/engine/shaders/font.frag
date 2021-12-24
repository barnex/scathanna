#version 300 es

precision mediump float;

in vec2 frag_tex_coord;
uniform vec3 color;
uniform sampler2D usampler;
out vec4 output_color;

void main() {
  vec4 rgba = texture(usampler, frag_tex_coord);
  output_color = vec4(rgba.rgb * color, rgba.a);
}