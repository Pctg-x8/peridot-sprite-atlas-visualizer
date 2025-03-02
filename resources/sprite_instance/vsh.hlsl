struct Output {
    float4 pos : SV_Position;
    float2 uv : TEXCOORD0;
};

struct RenderParams {
    float2 pixelSize;
    float2 offset;
} renderParams : register(c0);

float2 apply_st(in float2 base, in float4 st) {
    return base * st.xy + st.zw;
}

Output main(float2 base : POSITION0, float4 pos_st : POSITION1, float4 uv_st : TEXCOORD0) {
    const float2 xy = 2.0 * (apply_st(base, pos_st) - renderParams.offset) / renderParams.pixelSize - 1.0;

    Output o;
    o.uv = apply_st(base, uv_st);
    o.pos = float4(xy.x, -xy.y, 0.0, 1.0);

    return o;
}
