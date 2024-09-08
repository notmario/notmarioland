#version 150

in vec2 v_uv;
in vec4 v_color;

uniform sampler2D u_texture;
uniform sampler2D u_overlay;
uniform float time;
uniform float n_intensity;

out vec4 o_color;

void main() {
  float noise_val = fract(
    sin(dot(v_uv.xy ,vec2(12.9898,78.233 + time)) 
  ) * 43758.5453);
  o_color = v_color * texture(u_texture, v_uv) * (1.0 - n_intensity) + texture(u_texture, v_uv).a * vec4(noise_val, noise_val, noise_val, 1.0) * n_intensity;
  // o_color = v_color * texture(u_texture, v_uv);
}
