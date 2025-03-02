Texture2D tex : register(t0);
SamplerState smp : register(s0);

float4 main(float4 pos : SV_Position, float2 uv : TEXCOORD0) : SV_Target {
    float4 c = tex.Sample(smp, uv);
    c.rgb *= c.a;
    
    return c;
}
