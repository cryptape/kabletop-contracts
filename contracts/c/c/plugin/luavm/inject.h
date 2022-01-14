#ifndef CKB_LUA_LUAVM_INJECT
#define CKB_LUA_LUAVM_INJECT

#include "../inject.h"

void inject_luavm_functions(lua_State *L)
{
    inject_ckb_functions(L);
}

#endif