#version 300 es

layout(location = 0) in vec3 v_pos;
layout(location = 2) in vec2 v_tex;
uniform mat4 model;
uniform mat4 proj;

out vec2 f_tex;

void main() {
  gl_Position = proj * (model * vec4(v_pos, 1.0));
  f_tex = v_tex;
}