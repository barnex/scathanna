#version 300 es

layout(location = 0) in vec3 v_pos;
uniform mat4 proj;
uniform mat4 model;
uniform vec3 color;

out vec3 f_col;

void main() {
  gl_Position = proj * (model * vec4(v_pos, 1.0));
  f_col = color;
}