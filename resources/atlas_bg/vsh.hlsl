struct Output {
    float4 pos : SV_Position;
    float2 pixelCoord : TEXCOORD0;
};

struct RenderParams {
    float2 pixelSize;
    float2 offset;
} renderParams : register(c0);

Output main(float2 pixelCoord : POSITION0) {
    const float2 xy = 2.0 * (pixelCoord - renderParams.offset) / renderParams.pixelSize - 1.0;

    Output o;
    o.pixelCoord = pixelCoord;
    o.pos = float4(xy.x, -xy.y, 0.0, 1.0);

    return o;
}
