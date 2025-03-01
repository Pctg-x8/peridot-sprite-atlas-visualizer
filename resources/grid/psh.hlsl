struct Output {
    float4 pos : SV_Position;
    float2 uv : TEXCOORD0;
};

struct RenderParams {
    float2 pixelSize;
    float2 gridOffset;
    float gridSize;
} renderParams : register(c0);

float4 main(Output o) : SV_Target {
    // return float4(o.uv, 0.0f, 1.0f);

    const float2 uv1 = o.uv + renderParams.gridOffset / renderParams.pixelSize;
    const float2 lv0 = 1.0 - smoothstep(1.0 / renderParams.pixelSize, 2.0 / renderParams.pixelSize, abs(uv1));
    const float b0 = 1.0 - (1.0 - lv0.x) * (1.0 - lv0.y);

    const float2 div = renderParams.pixelSize / renderParams.gridSize;
    const float2 xr = abs(frac(uv1 * div) - 0.5) * 2.0;
    const float2 lv = smoothstep(1.0 - (1.0 / (renderParams.pixelSize / div / 2.0)), 1.0, xr);
    const float b = 1.0 - (1.0 - lv.x) * (1.0 - lv.y);

    return lerp(float4(0.1, 0.1, 0.2, 1.0), float4(0.5, 0.5, 0.5, 1.0), 1.0 - (1.0 - b) * (1.0 - b0));
}