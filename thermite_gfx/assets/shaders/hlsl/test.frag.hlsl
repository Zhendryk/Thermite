static float4 fragment_color;

struct SPIRV_Cross_Output
{
    float4 fragment_color : COLOR0;
};

void frag_main()
{
    fragment_color = float4(0.5f, 0.5f, 1.0f, 1.0f);
}

SPIRV_Cross_Output main()
{
    frag_main();
    SPIRV_Cross_Output stage_output;
    stage_output.fragment_color = float4(fragment_color);
    return stage_output;
}
