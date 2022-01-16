# kabletop-contracts

Consist of four contracts: nft, wallet, payment and kabletop
> details are here: https://talk.nervos.org/t/kabletop/5715

## NFT Contract

Act as a temporary simple NFT protocol for Kabletop which would be replaced by other official NFT protocols like Mibao soon.

NFT format announced in contract is just that one NFT represents 20 bytes which could be also called H160. More NFTs you have in Kabletop means more CKBs to be staked. All of staked CKBs can be regarded as TVL to Kabletop.

NFT can be also deleted and transferred to others, these two operations will release CKBs.

> Build contract:

``` sh
capsule build -n nft
```

> Run tests:

``` sh
cd tests/nft
capsule test -- --nocapture
```

## Wallet Contract

The wallet contract, also called the ownerlock contract, represents the NFT creator's wallet and implements logic similar to the ACP contract. This contract needs to be used in conjunction with the payment contract.

> Build contract:

``` sh
capsule build -n wallet
```

> Run tests:

``` sh
cd tests/wallet
capsule test -- --nocapture
```

## Payment Contract

Listen and handle purchase operation for the NFT owners. The payment contract should always be used in conjuction with the wallet contract.

When the owner of both the wallet and payment contracts is the creator of the NFT, the data represents the incremental rules set by the creator for the NFT, i.e., the output probability of each NFT.

When the owner of the wallet and payment contracts is the creator of the NFT and the owner of the NFT, respectively, the data represents the number of NFT card packs currently purchased by the owner. 

> Build contract:

 ``` sh
 capsule build -n payment
 ```

> Run tests:

``` sh
cd tests/payment
capsule test -- --nocapture
```

## Kabletop Contract

It's writen in C, and integrated the whole Lua interpretor engine to run lua codes which represent the GamePlay logic in CKB-VM.

The kabletop contract currently only supports two-player turn-based matchmaking scenarios, where both players sign each other's turn data, which will eventually be placed in the Witnesses field.

> Build contract:

 ``` sh
 capsule build -n kabletop
 ```

> Run tests:

``` sh
cd tests/kabletop
capsule test -- --nocapture
```