#ifndef CKB_LUA_PLUGIN
#define CKB_LUA_PLUGIN

#include "lauxlib.h"

#define CHECK_RET(x)    \
    ret = x;            \
    if (ret != 0) {		\
        return ret;     \
    }

int plugin_init(lua_State *L, int herr);

int plugin_verify(lua_State *L, int herr);

#endif
