#include "../plugin.h"
#include "inject.h"
#include "blockchain.h"
#include "core.h"
#include <stdio.h>

void hex(char *hex, uint8_t *bytes, int size)
{
	int pointer = 0;
	char hex_char[16];
	for (int i = 0; i < size; ++i)
	{
		sprintf(hex_char, "%02x", (int)bytes[i]);
		memcpy(&hex[pointer], hex_char, strlen(hex_char));
		pointer += strlen(hex_char);
	}
}

void import_user_nft(Kabletop *k, lua_State *L, _USER_NFT_F _user_nft, const char *name)
{
    lua_newtable(L);
	char hash[BLAKE160_SIZE * 2 + 1] = "";
    for (uint8_t i = 0; i < _user_deck_size(k); ++i)
    {
		uint8_t *nft = _user_nft(k, i);
		if (nft == NULL) break;
		hex(hash, nft, BLAKE160_SIZE);
        lua_pushstring(L, hash);
        lua_rawseti(L, -2, i + 1);
    }
    lua_setglobal(L, name);
}

int plugin_init(lua_State *L, int herr)
{
    luaL_openlibs(L);
    return inject_kabletop_functions(L, herr);
}

int plugin_verify(lua_State *L, int herr)
{
    // molecule buffers
    uint8_t script[MAX_SCRIPT_SIZE];
    uint8_t witnesses[MAX_ROUND_COUNT][MAX_ROUND_SIZE];
    uint8_t challenge_data[2][MAX_CHALLENGE_DATA_SIZE];

    Kabletop kabletop;
    int ret = CKB_SUCCESS;
    uint64_t capacities[3] = {0, 0, 0};

    // recover kabletop params from args
    CHECK_RET(verify_lock_args(&kabletop, script));

    // recover kabletop rounds from witnesses
    CHECK_RET(verify_witnesses(&kabletop, witnesses));

    // check challenge or settlement mode
    MODE mode = check_mode(&kabletop, challenge_data);
    switch (mode)
    {
        case MODE_SETTLEMENT: 
        {
            CHECK_RET(verify_settlement_mode(&kabletop, capacities));
            break;
        }
        case MODE_CHALLENGE:
        {
            CHECK_RET(verify_challenge_mode(&kabletop));
            break;
        }
        default: return KABLETOP_WRONG_MODE;
    }

    // import all users nft collection
    import_user_nft(&kabletop, L, _user1_nft, "_user1_nfts");
    import_user_nft(&kabletop, L, _user2_nft, "_user2_nfts");

	// load lua codes from celldep which match the hashes from kabletop_args
	CHECK_RET(inject_celldep_functions(&kabletop, L, herr));

    // check lua operations
    for (uint8_t i = 0; i < kabletop.round_count; ++i)
    {
        lua_getglobal(L, "_set_random_seed");
        lua_pushinteger(L, kabletop.seeds[i].randomseed[0]);
        lua_pushinteger(L, kabletop.seeds[i].randomseed[1]);
        lua_pcall(L, 2, 0, herr);
        uint8_t count = _operations_count(&kabletop, i);
        for (uint8_t n = 0; n < count; ++n)
        {
            Operation operation = _operation(&kabletop, i, n);
            if (luaL_loadbuffer(L, (const char *)operation.code, operation.size, "kabletop-running-operation")
                || lua_pcall(L, 0, 0, herr))
            {
				char error[512] = "";
				sprintf(error, "Invalid lua script: please check operation code [%u-%u].", i, n);
				ckb_debug(error);
                return KABLETOP_WRONG_LUA_OPERATION_CODE;
            }
        }
    }

    // check lua final state
    lua_getglobal(L, "_winner");
    int winner = lua_tointeger(L, -1);
    CHECK_RET(check_result(&kabletop, winner, capacities, mode));

    return CKB_SUCCESS;
}
