#version 410 core

layout(location = 0) in vec3 v_pos;
layout(location = 1) in vec3 v_col;
uniform mat4 model;
uniform mat4 proj;

out vec3 f_col;

void main() {
  gl_Position = proj * (model * vec4(v_pos, 1.0));
  f_col = v_col;
}