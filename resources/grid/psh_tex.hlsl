Texture2D tex : register(t0);

SamplerState ss {
    Filter = MIN_MAG_MIP_LINEAR;
    AddressU = Wrap;
    AddressV = Wrap;
};

float4 main(float4 pos : SV_Position, float2 uv : TEXCOORD0) : SV_Target {
    float4 c = tex.Sample(ss, uv);
    c.rgb *= c.a;
    
    return c;
}
