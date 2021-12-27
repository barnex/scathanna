/*
  Particle shader: computes the kinematics of free-falling particles on-the fly:

    vertex_pos(t) = vertex_pos(0) + vertex_velocity0 * time
                    - 1/2 * gravity * time^2
*/

#version 410 core

layout(location = 0) in vec3 v_pos;  // vertex position at time=0;
layout(location = 2) in vec2 v_tex;  // vertex texture coordiantes
layout(location = 4) in vec3 v_col;  // vertex color;
layout(location = 5) in vec3 v_vel0; // vertex velocity at time=0;
uniform float time;
uniform float gravity;
uniform mat4 model;
uniform mat4 proj;

out vec3 f_col;
out vec2 f_tex;

void main() {
  f_tex = v_tex;
  vec3 pos0 = (model * vec4(v_pos, 1.0)).xyz;
  vec3 pos = pos0 + v_vel0 * time;
  pos.y -= 0.5 * gravity * time * time;

  gl_Position = proj * vec4(pos, 1.0);
  f_col = v_col;
}