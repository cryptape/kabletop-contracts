#ifndef CKB_LUA_KABLETOP_INJECT
#define CKB_LUA_KABLETOP_INJECT

#include "../inject.h"
#include "core.h"
#include "luacode.c"

int inject_kabletop_functions(lua_State *L, int herr)
{
    inject_ckb_functions(L);

	// load internal code
    luaL_dostring(L, "                  \
        _winner = 0                     \
        function _set_random_seed(x, y) \
            math.randomseed(x, y)       \
        end                             \
    ");

	// load native code
    if (luaL_loadbuffer(L, (const char *)_GAME_CHUNK, _GAME_CHUNK_SIZE, "native")
        || lua_pcall(L, 0, 0, herr))
    {
        ckb_debug("Invalid lua script: please check native code.");
        return KABLETOP_WRONG_LUA_CONTEXT_CODE;
    }

    return CKB_SUCCESS;
}

int inject_celldep_functions(Kabletop *k, lua_State *L, int herr)
{
	// molecule buffers
	uint8_t data_hash[BLAKE2B_BLOCK_SIZE];
	uint8_t luacode[MAX_LUACODE_SIZE];

	uint8_t hashes_count = _lua_code_hashes_count(k);
	for (uint8_t h = 0; h < hashes_count; ++h)
	{
		bool matched = false;
		uint8_t *hash = _lua_code_hash(k, h);
		for (size_t i = 0; 1; ++i)
		{
			// check wether data_hash of celldep matches luacode_hash
			uint64_t size = BLAKE2B_BLOCK_SIZE;
			int ret = ckb_load_cell_by_field(data_hash, &size, 0, i, CKB_SOURCE_CELL_DEP, CKB_CELL_FIELD_DATA_HASH);
			if (ret == CKB_INDEX_OUT_OF_BOUND)
			{
				break;
			}
			if (ret != CKB_SUCCESS)
			{
				return KABLETOP_WRONG_LUA_CELLDEP_CODE;
			}
			if (memcmp(hash, data_hash, BLAKE2B_BLOCK_SIZE))
			{
				continue;
			}

			// load luacode from celldep data
			size = MAX_LUACODE_SIZE;
			ckb_load_cell_data(luacode, &size, 0, i, CKB_SOURCE_CELL_DEP);

			// load celldep code
			if (luaL_loadbuffer(L, (const char *)luacode, size, "celldep")
				|| lua_pcall(L, 0, 0, herr))
			{
				ckb_debug("Invalid lua script: please check celldep code.");
				return KABLETOP_WRONG_LUA_CELLDEP_CODE;
			}
			matched = true;
		}

		if (! matched)
		{
			return KABLETOP_WRONG_LUA_CELLDEP_CODE;
		}
	}

	return CKB_SUCCESS;
}

#endif