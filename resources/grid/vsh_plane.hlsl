struct Output {
    float4 pos : SV_Position;
    float2 uv : TEXCOORD0;
};

struct RenderParams {
    float2 pixelSize;
    float2 offset;
} renderParams : register(c0);

Output main(float2 uv_tex : TEXCOORD0) {
    const float2 xy = 2.0 * ((uv_tex * 4096.0) - renderParams.offset) / renderParams.pixelSize - 1.0;

    Output o;
    o.uv = uv_tex;
    o.pos = float4(xy.x, -xy.y, 0.0, 1.0);

    return o;
}
