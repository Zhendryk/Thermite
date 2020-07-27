#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct main0_out
{
    float4 fragment_color [[color(0)]];
};

fragment main0_out main0()
{
    main0_out out = {};
    out.fragment_color = float4(0.5, 0.5, 1.0, 1.0);
    return out;
}

