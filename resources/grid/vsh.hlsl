struct Output {
    float4 pos : SV_Position;
    float2 uv : TEXCOORD0;
};

Output main(uint vid : SV_VertexID) {
    const float2 xy = float2((vid & 0x01) == 0 ? 0.0f : 1.0f, (vid & 0x02) == 0 ? 0.0f : 1.0f);

    Output o;
    o.pos = float4(xy.x * 2.0f - 1.0f, -(xy.y * 2.0f - 1.0f), 0.0f, 1.0f);
    o.uv = xy;

    return o;
}
