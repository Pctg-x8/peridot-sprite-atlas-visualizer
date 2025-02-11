struct Output {
    float4 pos : SV_Position;
    float2 uv : TEXCOORD0;
};

struct RenderParams {
    float2 pixelSize;
    float gridSize;
} renderParams : register(c0);

float4 main(Output o) : SV_Target {
    // return float4(o.uv, 0.0f, 1.0f);

    const float2 div = renderParams.pixelSize / renderParams.gridSize;
    const float2 xr = abs(frac(o.uv * div) - 0.5) * 2.0;
    const float2 lv = smoothstep(1.0 - (1.0 / (renderParams.pixelSize / div / 2.0)), 1.0, xr);
    const float b = 1.0 - (1.0 - lv.x) * (1.0 - lv.y);
    return lerp(float4(0.1, 0.1, 0.2, 1.0), float4(0.5, 0.5, 0.5, 1.0), b);
}