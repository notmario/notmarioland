precision mediump float;
#define SPIN_EASE 0.8
#define spin_time 5.
#define spin_amount 0.3
#define contrast 3.

#define colour_1 vec4(0.6392156863,0.3294117647,1.0,1.0)
#define colour_2 vec4(0.7960784314,0.0,1.0,1.0)
#define colour_3 vec4(0.2156862745,0.2588235294,0.2666666667,1.0)

uniform vec2 iResolution;
uniform float iTime;
varying vec2 uv;

void main( )
{
    // uncomment to enable pixelated bg
    float pixel_size = 1.0;
    vec2 uv = (floor((uv * iResolution.xy)*(1./pixel_size))*pixel_size - 0.5*iResolution.xy)/length(iResolution.xy) - vec2(0.12, 0.);
    
    float uv_len = length(uv);
    
    float speed = (spin_time*SPIN_EASE*0.2) + 281.7;
    float new_angle = (atan(uv.y, uv.x)) + speed - SPIN_EASE*20.*(1.*spin_amount*uv_len + (1. - 1.*spin_amount));
    
    vec2 middle = vec2(0.5, 0.5);
    uv = (vec2((uv_len * cos(new_angle) + middle.x), (uv_len * sin(new_angle) + middle.y)) - middle);
    
    uv *= 30.;
    speed = iTime*(2.);
    vec2 uv2 = vec2(uv.x+uv.y);

    for(int i=0; i < 6; i++) {
        uv2 += sin(max(uv.x, uv.y)) + uv;
        uv  += 0.5*vec2(cos(4.91254 + 0.353*uv2.y + speed*0.089283),sin(uv2.x - 0.098*speed));
        uv  -= 1.0*cos(uv.x + uv.y) - 1.0*sin(uv.x*0.711 - uv.y);
    }
    
    float contrast_mod = (0.25*contrast + 0.5*spin_amount + 1.2);
    float paint_res =min(2., max(0.,length(uv)*(0.035)*contrast_mod));
    float c1p = max(0.,1. - contrast_mod*abs(1.-paint_res));
    float c2p = max(0.,1. - contrast_mod*abs(paint_res));
    float c3p = 1. - min(1., c1p + c2p);

    // Output to screen
    vec4 outcol = (0.3/contrast)*colour_1 + (1. - 0.3/contrast)*(colour_1*c1p + colour_2*c2p + vec4(c3p*colour_3.rgb, c3p*colour_1.a));

    gl_FragColor = outcol;
}
