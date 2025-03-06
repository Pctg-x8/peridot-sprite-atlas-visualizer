float v(in float2 pixelCoordInt) {
    const float2 r = trunc(pixelCoordInt / 16.0) % 2.0;

    return r.x == r.y ? 1.0 : 0.0;
}

float4 main(float4 pos : SV_Position, float2 pixelCoord : TEXCOORD0) : SV_Target {
    const float2 f = frac(pixelCoord);
    const float v00 = v(trunc(pixelCoord));
    const float v10 = v(trunc(pixelCoord + float2(1.0, 0.0)));
    const float v01 = v(trunc(pixelCoord + float2(0.0, 1.0)));
    const float v11 = v(trunc(pixelCoord + 1.0));

    const float v = lerp(lerp(v00, v10, f.x), lerp(v01, v11, f.x), f.y);
    return lerp(float4(0.75, 0.75, 0.75, 1.0), float4(0.5, 0.5, 0.5, 1.0), v);
}
