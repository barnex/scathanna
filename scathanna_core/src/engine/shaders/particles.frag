#version 410 core

precision highp float;

in vec3 f_col;
in vec2 f_tex;
uniform vec4 color;
out vec4 out_col;

void main() {
  float r2 = dot(f_tex, f_tex);
  if (r2 < 0.05) { // TODO: squared radius of circle enscribed
                   // in unit equilateral triangle
    out_col = vec4(f_col, 1.0) * color;
  } else {
    discard;
  }
}