#include "../plugin.h"
#include "inject.h"
#include "blockchain.h"

#define MAX_WITNESS_SIZE 32768
#define MAX_SCRIPT_SIZE 32768
#define ERROR_LOADING_SCRIPT 4

int plugin_init(lua_State *L, int herr)
{
    luaL_openlibs(L);
    inject_luavm_functions(L);

    return 0;
}

int plugin_verify(lua_State *L, int herr)
{
    // Fetch ckb script from context and point to "args" field
    unsigned char script[MAX_SCRIPT_SIZE];
    uint64_t len = MAX_SCRIPT_SIZE;
    int ret = ckb_load_script(script, &len, 0);
    if (ret != CKB_SUCCESS || len > MAX_SCRIPT_SIZE)
    {
        return ERROR_LOADING_SCRIPT;
    }
    mol_seg_t script_seg;
    script_seg.ptr = (uint8_t *)script;
    script_seg.size = len;
    if (MolReader_Script_verify(&script_seg, false) != MOL_OK)
    {
        return ERROR_LOADING_SCRIPT;
    }
    mol_seg_t args_seg = MolReader_Script_get_args(&script_seg);
    mol_seg_t args_bytes_seg = MolReader_Bytes_raw_bytes(&args_seg);
    if (args_bytes_seg.size > MAX_SCRIPT_SIZE)
    {
        return ERROR_LOADING_SCRIPT;
    }

    // Run lua code from typescript's args
    if (luaL_loadbuffer(L, (const char *)args_bytes_seg.ptr, args_bytes_seg.size, "luavm")
        || lua_pcall(L, 0, 0, herr))
    {
        ckb_debug("Invalid lua script: please check your lua code.");
        return ERROR_LOADING_SCRIPT;
    }

    return 0;
}
