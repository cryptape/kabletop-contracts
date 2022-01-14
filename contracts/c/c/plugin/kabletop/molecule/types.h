#ifndef CKB_KABLETOP_TYPES
#define CKB_KABLETOP_TYPES

#include "kabletop.h"

#define MAX_ROUND_COUNT 256

typedef enum
{
    USER_KABLETOP,
    USER_1,
    USER_2,
} USER_TYPE;

typedef struct
{
    uint8_t  size;
    uint8_t *code;
} Operation;

typedef struct
{
    uint64_t randomseed[2];
} Seed;

typedef struct
{
    // from input lock_args
    mol_seg_t args;

    // from witnesses
    uint8_t round_count;
    mol_seg_t rounds[MAX_ROUND_COUNT];
	mol_seg_t signatures[MAX_ROUND_COUNT];

    // from data
    mol_seg_t input_challenge;
    mol_seg_t output_challenge;

    // others
    Seed seeds[MAX_ROUND_COUNT];
    USER_TYPE signer;
} Kabletop;

#define _user_staking_ckb(k)       *(uint64_t *)MolReader_Args_get_user_staking_ckb(&k->args).ptr
#define _user_deck_size(k)         *(uint8_t *)MolReader_Args_get_user_deck_size(&k->args).ptr
#define _begin_blocknumber(k)      *(uint64_t *)MolReader_Args_get_begin_blocknumber(&k->args).ptr
#define _lock_code_hash(k)          (uint8_t *)MolReader_Args_get_lock_code_hash(&k->args).ptr
#define _user1_pkhash(k)            (uint8_t *)MolReader_Args_get_user1_pkhash(&k->args).ptr
#define _user2_pkhash(k)            (uint8_t *)MolReader_Args_get_user2_pkhash(&k->args).ptr
#define _user_type(k, i)           *(uint8_t *)MolReader_Round_get_user_type(&k->rounds[i]).ptr
#define _challenger(k, io)         *(uint8_t *)MolReader_Challenge_get_challenger(&k->io##_challenge).ptr
#define _snapshot_position(k, io)  *(uint8_t *)MolReader_Challenge_get_snapshot_position(&k->io##_challenge).ptr
#define _snapshot_hashproof(k, io)  (uint8_t *)MolReader_Challenge_get_snapshot_hashproof(&k->io##_challenge).ptr
#define _snapshot_signature(k, io)  (uint8_t *)MolReader_Challenge_get_snapshot_signature(&k->io##_challenge).ptr
#define _challenge_count(k, io)    *(uint8_t *)MolReader_Challenge_get_count(&k->io##_challenge).ptr

uint8_t _lua_code_hashes_count(Kabletop *k)
{
    mol_seg_t hashes = MolReader_Args_get_lua_code_hashes(&k->args);
	return (uint8_t)MolReader_Hashes_length(&hashes);
}

uint8_t * _lua_code_hash(Kabletop *k, uint8_t i)
{
    mol_seg_t hashes = MolReader_Args_get_lua_code_hashes(&k->args);
	if (i < MolReader_Hashes_length(&hashes))
	{
		mol_seg_t hash =  MolReader_Hashes_get(&hashes, i).seg;
		return (uint8_t *)hash.ptr;
	}
	return NULL;
}

uint8_t * _user1_nft(Kabletop *k, uint8_t i)
{
    mol_seg_t nfts = MolReader_Args_get_user1_nfts(&k->args);
    if (i < MolReader_nfts_length(&nfts))
    {
        mol_seg_res_t nft = MolReader_nfts_get(&nfts, i);
        return (uint8_t *)nft.seg.ptr;
    }
    return NULL;
}

uint8_t * _user2_nft(Kabletop *k, uint8_t i)
{
    mol_seg_t nfts = MolReader_Args_get_user2_nfts(&k->args);
    if (i < MolReader_nfts_length(&nfts))
    {
        mol_seg_res_t nft = MolReader_nfts_get(&nfts, i);
        return (uint8_t *)nft.seg.ptr;
    }
    return NULL;
}

uint8_t _operations_count(Kabletop *k, uint8_t i)
{
    mol_seg_t operations = MolReader_Round_get_operations(&k->rounds[i]);
    return (uint8_t)MolReader_Operations_length(&operations);
}

Operation _operation(Kabletop *k, uint8_t r, uint8_t i)
{
    mol_seg_t operation = MolReader_Round_get_operations(&k->rounds[r]);
    operation = MolReader_Operations_get(&operation, i).seg;
    Operation op;
    op.size = (uint8_t)MolReader_bytes_length(&operation);
    op.code = (uint8_t *)MolReader_bytes_raw_bytes(&operation).ptr;
    return op;
}

uint8_t _input_challenge_operations_count(Kabletop *k)
{
	mol_seg_t operations = MolReader_Challenge_get_operations(&k->input_challenge);
	return (uint8_t)MolReader_Operations_length(&operations);
}

uint8_t _output_challenge_operations_count(Kabletop *k)
{
	mol_seg_t operations = MolReader_Challenge_get_operations(&k->output_challenge);
	return (uint8_t)MolReader_Operations_length(&operations);
}

typedef uint8_t * _USER_NFT_F(Kabletop *, uint8_t);

#endif
