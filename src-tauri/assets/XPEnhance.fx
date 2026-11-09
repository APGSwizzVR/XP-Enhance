#include "ReShade.fxh"

uniform float Exposure < ui_label = "Exposure"; ui_type = "slider"; ui_min = -2.0; ui_max = 2.0; ui_step = 0.01; > = 0.0;
uniform float Contrast < ui_label = "Contrast"; ui_type = "slider"; ui_min = 0.5; ui_max = 1.8; ui_step = 0.01; > = 1.0;
uniform float Saturation < ui_label = "Saturation"; ui_type = "slider"; ui_min = 0.0; ui_max = 2.0; ui_step = 0.01; > = 1.0;
uniform float Warmth < ui_label = "Warmth"; ui_type = "slider"; ui_min = -0.25; ui_max = 0.25; ui_step = 0.005; > = 0.0;

float4 PS_XPEnhance(float4 position : SV_Position, float2 texcoord : TEXCOORD) : SV_Target
{
    float4 colour = tex2D(ReShade::BackBuffer, texcoord);
    colour.rgb *= exp2(Exposure);
    colour.rgb = (colour.rgb - 0.5) * Contrast + 0.5;
    float luminance = dot(colour.rgb, float3(0.2126, 0.7152, 0.0722));
    colour.rgb = lerp(luminance.xxx, colour.rgb, Saturation);
    colour.r += Warmth;
    colour.b -= Warmth;
    return saturate(colour);
}

technique XPEnhance_FlightGrade
{
    pass
    {
        VertexShader = PostProcessVS;
        PixelShader = PS_XPEnhance;
    }
}
