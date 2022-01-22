#version 410 core

precision mediump float;

in vec3 f_normal;
in vec3 f_pos;
in highp vec2 f_tex;
out vec4 out_col;
uniform sampler2D usampler;
uniform vec3 sun_dir;
uniform float sun_intens;
uniform float ambient;
uniform vec3 cam_pos;

void main() {

  vec3 normal = normalize(f_normal);

  float costheta = dot(sun_dir, normal);
  float direct = sun_intens * max(0.0, costheta);

  float dir_ambient = ambient * (0.5 * (costheta + 2.0)); // TODO

  float diffuse = direct + dir_ambient;

  vec3 view_dir = normalize(cam_pos - f_pos);
  vec3 refl_dir = reflect(normalize(-sun_dir), normal);
  float specular = 0.3 * sun_intens * pow(max(0.0, dot(view_dir, refl_dir)), 8);

  vec3 tex = texture(usampler, f_tex).rgb;
  vec3 color = diffuse * tex + vec3(specular, specular, specular);
  out_col = vec4(color, 1.0);
}
